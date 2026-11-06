//! Tables that read on a phone.
//!
//! A table with two columns lists terms and their meanings, so it becomes a definition list.
//! A wider table keeps its grid on wide screens. Each cell has the label of its column,
//! so on a narrow screen each row becomes a card.

use std::mem;

use maud::{PreEscaped, html};
use pulldown_cmark::{Event, Tag, TagEnd};

/// Replaces each table in the event stream with one HTML event.
pub fn transform(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut out = Vec::with_capacity(events.len());
    let mut events = events.into_iter();
    while let Some(event) = events.next() {
        if !matches!(event, Event::Start(Tag::Table(_))) {
            out.push(event);
            continue;
        }
        let mut table = Table::default();
        for event in events.by_ref() {
            if matches!(event, Event::End(TagEnd::Table)) {
                break;
            }
            table.push(event);
        }
        out.push(Event::Html(table.html().into()));
    }
    out
}

#[derive(Default)]
struct Table<'a> {
    head: Vec<Cell>,
    rows: Vec<Vec<Cell>>,
    row: Vec<Cell>,
    cell: Vec<Event<'a>>,
    label: String,
    in_head: bool,
}

/// One rendered cell and its plain text.
struct Cell {
    html: String,
    text: String,
}

impl<'a> Table<'a> {
    fn push(&mut self, event: Event<'a>) {
        match event {
            Event::Start(Tag::TableHead) => self.in_head = true,
            Event::End(TagEnd::TableHead) => self.in_head = false,
            Event::Start(Tag::TableRow) => self.row.clear(),
            Event::End(TagEnd::TableRow) if !self.in_head => {
                self.rows.push(mem::take(&mut self.row));
            }
            Event::Start(Tag::TableCell) => {
                self.cell.clear();
                self.label.clear();
            }
            Event::End(TagEnd::TableCell) => {
                let mut html = String::new();
                pulldown_cmark::html::push_html(&mut html, self.cell.drain(..));
                let cell = Cell {
                    html,
                    text: mem::take(&mut self.label),
                };
                if self.in_head {
                    self.head.push(cell);
                } else {
                    self.row.push(cell);
                }
            }
            event => {
                if let Event::Text(text) | Event::Code(text) = &event {
                    self.label.push_str(text);
                }
                self.cell.push(event);
            }
        }
    }

    fn html(&self) -> String {
        if self.head.len() == 2 {
            return self.definitions();
        }
        html! {
            div.grid-table { table {
                thead { tr { @for cell in &self.head { th scope="col" { (PreEscaped(&cell.html)) } } } }
                tbody { @for row in &self.rows { tr {
                    th scope="row" { (PreEscaped(cell_html(row, 0))) }
                    @for (index, head) in self.head.iter().enumerate().skip(1) {
                        td data-label=(head.text.trim()) { span { (PreEscaped(cell_html(row, index))) } }
                    }
                } } }
            } }
        }
        .into_string()
    }

    fn definitions(&self) -> String {
        html! {
            dl.term-list { @for row in &self.rows {
                div { dt { (PreEscaped(cell_html(row, 0))) } dd { (PreEscaped(cell_html(row, 1))) } }
            } }
        }
        .into_string()
    }
}

fn cell_html(row: &[Cell], index: usize) -> &str {
    row.get(index).map_or("", |cell| cell.html.as_str())
}
