//! Field assignments from the command line, such as `title=Notes` and `brand.credit:=false`.

use std::str::FromStr;

use serde_json::{Map, Value};

/// One field assignment: `path=text` sets a string, `path:=json` sets any JSON value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    path: Vec<String>,
    value: Value,
}

/// An assignment that cannot be parsed or applied.
#[derive(Debug, thiserror::Error)]
pub enum AssignmentError {
    #[error("{0:?} is not an assignment: use field=text or field:=json")]
    Syntax(String),
    #[error("{field}: the value after := is not valid JSON")]
    Json {
        field: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("{0}: no such field")]
    Path(String),
}

impl Assignment {
    /// Returns the dotted field path, such as `brand.accent`.
    #[must_use]
    pub fn field(&self) -> String {
        self.path.join(".")
    }

    /// Sets the field inside `target`.
    ///
    /// Objects on the path get missing keys, so a wrong name fails later as an unknown field.
    /// A numeric segment selects an existing array item.
    ///
    /// # Errors
    ///
    /// Returns [`AssignmentError::Path`] when the path goes through a value that is not an object or an array.
    pub fn apply(&self, target: &mut Value) -> Result<(), AssignmentError> {
        let Some((last, parents)) = self.path.split_last() else {
            return Err(AssignmentError::Path(self.field()));
        };
        let mut current = target;
        for segment in parents {
            current = match current {
                Value::Object(map) => map
                    .entry(segment.clone())
                    .or_insert_with(|| Value::Object(Map::new())),
                Value::Array(items) => item(items, segment).ok_or_else(|| self.path_error())?,
                _ => return Err(self.path_error()),
            };
        }
        match current {
            Value::Object(map) => {
                map.insert(last.clone(), self.value.clone());
            }
            Value::Array(items) => {
                *item(items, last).ok_or_else(|| self.path_error())? = self.value.clone();
            }
            _ => return Err(self.path_error()),
        }
        Ok(())
    }

    fn path_error(&self) -> AssignmentError {
        AssignmentError::Path(self.field())
    }
}

impl FromStr for Assignment {
    type Err = AssignmentError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let syntax = || AssignmentError::Syntax(text.to_owned());
        let (key, raw) = text.split_once('=').ok_or_else(syntax)?;
        let (key, value) = match key.strip_suffix(':') {
            Some(key) => {
                let value = serde_json::from_str(raw).map_err(|source| AssignmentError::Json {
                    field: key.to_owned(),
                    source,
                })?;
                (key, value)
            }
            None => (key, Value::from(raw)),
        };
        let path: Vec<String> = key.split('.').map(str::to_owned).collect();
        if path.iter().any(String::is_empty) {
            return Err(syntax());
        }
        Ok(Self { path, value })
    }
}

/// Returns the array item at the index in `segment`, if it exists.
fn item<'a>(items: &'a mut [Value], segment: &str) -> Option<&'a mut Value> {
    items.get_mut(segment.parse::<usize>().ok()?)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::assign::{Assignment, AssignmentError};

    fn apply(assignment: &str, mut target: serde_json::Value) -> serde_json::Value {
        assignment
            .parse::<Assignment>()
            .unwrap()
            .apply(&mut target)
            .unwrap();
        target
    }

    #[test]
    fn equals_sets_text_and_colon_equals_sets_json() {
        let target = json!({"title": "Old", "brand": {"credit": true}});
        assert_eq!(apply("title=true", target.clone())["title"], json!("true"));
        assert_eq!(
            apply("brand.credit:=false", target.clone())["brand"]["credit"],
            json!(false)
        );
        assert_eq!(
            apply("brand.logo:=null", target)["brand"]["logo"],
            json!(null)
        );
    }

    #[test]
    fn text_value_keeps_later_equals_signs() {
        let target = apply("cta=a=b", json!({}));
        assert_eq!(target["cta"], json!("a=b"));
    }

    #[test]
    fn numeric_segment_replaces_an_existing_item() {
        let target = json!({"aliases": ["a.test", "b.test"]});
        assert_eq!(
            apply("aliases.1=c.test", target)["aliases"],
            json!(["a.test", "c.test"])
        );
    }

    #[test]
    fn missing_item_and_scalar_parent_are_errors() {
        let mut target = json!({"aliases": ["a.test"], "title": "Notes"});
        for text in ["aliases.3=x", "title.size=1"] {
            let error = text
                .parse::<Assignment>()
                .unwrap()
                .apply(&mut target)
                .unwrap_err();
            assert!(matches!(error, AssignmentError::Path(_)), "{text}");
        }
    }

    #[test]
    fn malformed_assignments_are_rejected() {
        for text in ["title", "=x", "brand..accent=x", "a.=x"] {
            assert!(
                matches!(text.parse::<Assignment>(), Err(AssignmentError::Syntax(_))),
                "{text}"
            );
        }
        assert!(matches!(
            "tags:=[oops".parse::<Assignment>(),
            Err(AssignmentError::Json { .. })
        ));
    }
}
