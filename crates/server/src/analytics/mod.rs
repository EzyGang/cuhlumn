//! Page-view and link-click analytics: counts in memory, written to the database once a minute.

pub mod agent;
pub mod pending;
pub mod referrer;
pub mod unverified;
pub mod visitor;

use std::{
    mem,
    net::IpAddr,
    sync::{Mutex, PoisonError},
    time::Duration,
};

use cuhlumn_api::{
    analytics::ClickKind,
    site::{host::Host, kind::SiteKind},
};
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

use crate::{
    analytics::{
        agent::Client,
        pending::{Counted, Pending},
        visitor::{Salt, day_of, visitor_hash},
    },
    db::{self, Database, DbError, analytics::StoredSalt, unix_now},
    state::AppState,
};

/// The page script, which sends one `POST /_hit` for each page view and each counted link click.
///
/// Blog pages contain it inline, and external sites load it from `/_hit.js`.
pub const SCRIPT: &str = include_str!("../../assets/hit.js");

/// The page views and link clicks that wait for the database, and the visitor salt of the current day.
pub struct Analytics {
    pending: Mutex<Pending>,
    salt: Mutex<Salt>,
}

/// A page view that passed the host and page check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    /// The canonical site host.
    pub host: Host,
    /// The kind of the site.
    pub site: SiteKind,
    /// The decoded page path, such as `/sqlite-backups/`.
    pub path: String,
    /// The referring host, or `None` for a direct visit.
    pub referrer: Option<String>,
    pub client: Client,
    /// The client address, or `None` when the request has none.
    pub ip: Option<IpAddr>,
    pub user_agent: String,
}

/// A link click that passed the host check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Click {
    /// The canonical site host.
    pub host: Host,
    pub kind: ClickKind,
    /// The host and the path of the link, such as `github.com/EzyGang/cuhlumn`.
    pub target: String,
}

/// The time between two database writes.
const FLUSH_EVERY: Duration = Duration::from_secs(60);

impl Analytics {
    /// Starts with the stored salt when it belongs to the day of `now`, else with a new salt.
    ///
    /// # Errors
    ///
    /// Returns [`getrandom::Error`] when the operating system random source fails.
    pub fn new(stored: Option<StoredSalt>, now: i64) -> Result<Self, getrandom::Error> {
        Ok(Self {
            pending: Mutex::new(Pending::default()),
            salt: Mutex::new(Salt::restore(stored, day_of(now))?),
        })
    }

    /// Adds a checked page view at the unix time `now`.
    ///
    /// A failure of the random source drops the view, so a salt never covers two days.
    pub fn record(&self, view: View, now: i64) {
        let day = day_of(now);
        let Some(visitor) = self.with_salt(day, |salt| {
            visitor_hash(salt, view.host.as_str(), view.ip, &view.user_agent)
        }) else {
            return;
        };
        let counted = Counted {
            site: view.site,
            path: view.path,
            referrer: view.referrer,
            device: view.client.device,
            browser: view.client.browser,
            visitor,
        };
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .add(&view.host, day, counted);
    }

    /// Adds a checked link click at the unix time `now`.
    ///
    /// A click is not a page view, so it changes no view or visitor count.
    pub fn record_click(&self, click: Click, now: i64) {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .add_click(&click.host, day_of(now), click.kind, click.target);
    }

    /// Writes the waiting page views, deletes the visitor hashes of past days, and stores the salt of today.
    ///
    /// A failed write logs the error and drops the batch, because analytics are best effort.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] when the write fails.
    pub async fn flush(&self, db: &Database, now: i64) -> Result<(), DbError> {
        let Some(salt) = self.with_salt(day_of(now), Salt::clone) else {
            return Ok(());
        };
        let batch = mem::take(&mut *self.pending.lock().unwrap_or_else(PoisonError::into_inner));
        let views = batch.views();
        let written = db
            .call(move |connection| db::analytics::write::flush(connection, &batch, &salt, now))
            .await;
        if let Err(err) = &written {
            tracing::error!(views, error = %err, "analytics write failed, the batch is dropped");
        }
        written
    }

    /// Writes the waiting page views every minute, and once more when `stop` is cancelled.
    pub async fn run(state: AppState, stop: CancellationToken) {
        let mut ticker = tokio::time::interval(FLUSH_EVERY);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        ticker.tick().await;
        loop {
            tokio::select! {
                _ = ticker.tick() => (),
                () = stop.cancelled() => break,
            }
            // The flush logs its own failure.
            state.analytics().flush(state.db(), unix_now()).await.ok();
        }
        state.analytics().flush(state.db(), unix_now()).await.ok();
    }

    /// Runs `f` with the salt of `day`, and replaces the salt first when the day changed.
    ///
    /// Returns `None` when a new salt cannot be made.
    fn with_salt<F, T>(&self, day: i64, f: F) -> Option<T>
    where
        F: FnOnce(&Salt) -> T,
    {
        let mut salt = self.salt.lock().unwrap_or_else(PoisonError::into_inner);
        if let Err(err) = salt.rotate(day) {
            tracing::error!(error = %err, "the analytics salt cannot rotate");
            return None;
        }
        Some(f(&salt))
    }
}
