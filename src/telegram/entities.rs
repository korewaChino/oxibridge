use markdown::{mdast::Node, ParseOptions};
use teloxide::types::MessageEntity;

#[derive(Debug, PartialEq, Eq)]
pub struct StringWithEntities(pub String, pub Vec<MessageEntity>);

impl StringWithEntities {
    fn new() -> Self {
        Self(String::new(), vec![])
    }

    fn join(&mut self, other: &Self) {
        // FIXME: this will break if we have any unicode characters because i'm lazy rn
        let offset_entities = other
            .1
            .iter()
            .map(|x| MessageEntity {
                kind: x.kind.clone(),
                offset: x.offset + self.0.len(),
                length: x.length,
            })
            .collect::<Vec<MessageEntity>>();

        self.0.push_str(&other.0);
        self.1.extend(offset_entities);
    }

    fn join_strings(strings: Vec<Self>) -> Self {
        let mut string = Self::new();

        for other in strings {
            string.join(&other);
        }

        string
    }
}

fn nodes_to_entities(nodes: Vec<Node>) -> StringWithEntities {
    StringWithEntities::join_strings(nodes.iter().map(|node| node_to_entities(node)).collect())
}

fn node_to_entities(node: &Node) -> StringWithEntities {
    match node {
        Node::Root(root) => nodes_to_entities(root.children.clone()),
        Node::Paragraph(root) => nodes_to_entities(root.children.clone()),

        Node::Text(text) => StringWithEntities(text.value.clone(), vec![]),

        Node::Strong(strong) => {
            let string = nodes_to_entities(strong.children.clone());

            let entity = MessageEntity::bold(0, string.0.len());
            let entities = [entity].into_iter().chain(string.1.into_iter()).collect();

            StringWithEntities(string.0.clone(), entities)
        }

        Node::Emphasis(em) => {
            let string = nodes_to_entities(em.children.clone());

            let entity = MessageEntity::italic(0, string.0.len());
            let entities = [entity].into_iter().chain(string.1.into_iter()).collect();

            StringWithEntities(string.0.clone(), entities)
        }

        Node::InlineCode(node) => StringWithEntities(
            node.value.clone(),
            vec![MessageEntity::code(0, node.value.len())],
        ),

        Node::Code(node) => StringWithEntities(
            node.value.clone(),
            vec![MessageEntity::pre(node.lang.clone(), 0, node.value.len())],
        ),

        _ => StringWithEntities(format!("unknown node {node:?}"), vec![]),
    }
}

pub fn to_string_with_entities(value: &str) -> StringWithEntities {
    let node = markdown::to_mdast(value, &ParseOptions::default()).unwrap();
    println!("node={node:#?}");
    node_to_entities(&node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bold_italic_correctly() {
        let string = "hello, _**world**_!";

        assert_eq!(
            to_string_with_entities(string),
            StringWithEntities(
                "hello, world!".to_owned(),
                vec![MessageEntity::italic(7, 5), MessageEntity::bold(7, 5)]
            )
        );
    }

    #[test]
    fn leaves_newlines_as_is() {
        let string = "hello\nworld";

        assert_eq!(
            to_string_with_entities(string),
            StringWithEntities("hello\nworld".to_owned(), vec![])
        );
    }

    #[test]
    fn parses_code_correctly() {
        let string = "```rs
println!(\"hello, world!\");
```";

        assert_eq!(
            to_string_with_entities(string),
            StringWithEntities(
                "println!(\"hello, world!\");".to_owned(),
                vec![MessageEntity::pre(Some("rs".to_owned()), 0, 26)]
            )
        );
    }
}