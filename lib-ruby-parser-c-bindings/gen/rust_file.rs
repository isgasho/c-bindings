use super::CField;
use lib_ruby_parser_nodes::{FieldType, Node};

pub struct RustFile {
    nodes: Vec<Node>,
}

impl RustFile {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    pub fn code(&self) -> String {
        format!(
            "use crate::bindings::*;
use crate::StringPtr;
use crate::ptr_value;

impl From<lib_ruby_parser::Node> for Node {{
    fn from(node: lib_ruby_parser::Node) -> Self {{
        match node {{
{branches}
        }}
    }}
}}

{from_impls}
",
            branches = self.branches(),
            from_impls = self
                .nodes
                .iter()
                .map(|node| FromImplementation::new(node).code())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn branches(&self) -> String {
        self.nodes
            .iter()
            .map(|node| {
                format!(
                    "            lib_ruby_parser::Node::{name}(inner) => Node::from(inner),",
                    name = node.struct_name
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

struct FromImplementation<'a> {
    node: &'a Node,
}

impl<'a> FromImplementation<'a> {
    pub fn new(node: &'a Node) -> Self {
        Self { node }
    }

    pub fn code(&self) -> String {
        format!(
            "
impl From<lib_ruby_parser::nodes::{name}> for Node {{
    fn from(node: lib_ruby_parser::nodes::{name}) -> Self {{
        let node_type = NodeType_NODE_{enum_name};
{cast_fields}
        let typed_node = {typed_node_name} {{ {fields_list} }};
        let inner = InnerNode {{ _{union_field_name}: ptr_value(typed_node) }};
        Node {{ node_type, inner }}
    }}
}}
",
            name = self.node.struct_name,
            enum_name = self.node.filename.to_uppercase(),
            cast_fields = self.cast_fields(),
            typed_node_name = self.node.struct_name,
            fields_list = self.fields_list(),
            union_field_name = self.node.filename.to_lowercase(),
        )
    }

    pub fn cast_fields(&self) -> String {
        self.node
            .fields
            .iter()
            .map(|field| {
                let field_name = CField::new(field).field_name();

                let get = match field.field_type {
                    FieldType::Node => format!("ptr_value(Node::from(node.{}))", field.field_name),
                    FieldType::Nodes => format!("ptr_value(NodeList::from(node.{}))", field.field_name),
                    FieldType::MaybeNode => format!("if let Some(v) = node.{} {{ ptr_value(Node::from(v)) }} else {{ std::ptr::null_mut() }}", field.field_name),
                    FieldType::Range => format!("ptr_value(Range::from(node.{}))", field.field_name),
                    FieldType::MaybeRange => format!("if let Some(v) = node.{} {{ ptr_value(Range::from(v)) }} else {{ std::ptr::null_mut() }}", field.field_name),
                    FieldType::Str => {
                        format!("StringPtr::from(node.{}).unwrap()", field.field_name)
                    }
                    FieldType::MaybeStr => {
                        format!("StringPtr::from(node.{}).unwrap()", field.field_name)
                    }
                    FieldType::Chars => {
                        format!("StringPtr::from(node.{}).unwrap()", field.field_name)
                    }
                    FieldType::StringValue => {
                        format!("StringPtr::from(node.{}).unwrap()", field.field_name)
                    }
                    FieldType::U8 => format!("node.{} as size_t", field.field_name),
                    FieldType::Usize => format!("node.{} as size_t", field.field_name),
                    FieldType::RawString => {
                        format!("StringPtr::from(node.{}).unwrap()", field.field_name)
                    }
                    FieldType::RegexOptions => format!("if let Some(v) = node.{} {{ ptr_value(Node::from(v)) }} else {{ std::ptr::null_mut() }}", field.field_name),
                };
                format!(
                    "        let {field_name} = {get};",
                    field_name = field_name,
                    get = get
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn fields_list(&self) -> String {
        self.node
            .fields
            .iter()
            .map(|field| CField::new(field).field_name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}