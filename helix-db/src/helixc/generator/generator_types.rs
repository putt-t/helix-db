use core::fmt;
use std::fmt::Display;

use crate::{helix_engine::graph_core::config::Config, helixc::parser::helix_parser::FieldPrefix};

use super::{
    traversal_steps::Traversal,
    tsdisplay::ToTypeScript,
    utils::{GenRef, GeneratedType, GeneratedValue, write_headers},
};

pub struct Source {
    pub nodes: Vec<NodeSchema>,
    pub edges: Vec<EdgeSchema>,
    pub vectors: Vec<VectorSchema>,
    pub queries: Vec<Query>,
    pub config: Config,
    pub src: String,
}
impl Default for Source {
    fn default() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            vectors: vec![],
            queries: vec![],
            config: Config::default(),
            src: "".to_string(),
        }
    }
}
impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", write_headers())?;
        writeln!(f, "{}", self.config)?;
        write!(
            f,
            "{}",
            self.nodes
                .iter()
                .map(|n| format!("{n}"))
                .collect::<Vec<_>>()
                .join("\n")
        )?;
        writeln!(f)?;
        write!(
            f,
            "{}",
            self.edges
                .iter()
                .map(|e| format!("{e}"))
                .collect::<Vec<_>>()
                .join("\n")
        )?;
        writeln!(f)?;
        write!(
            f,
            "{}",
            self.vectors
                .iter()
                .map(|v| format!("{v}"))
                .collect::<Vec<_>>()
                .join("\n")
        )?;
        writeln!(f)?;
        write!(
            f,
            "{}",
            self.queries
                .iter()
                .map(|q| format!("{q}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Clone)]
pub struct NodeSchema {
    pub name: String,
    pub properties: Vec<SchemaProperty>,
}
impl Display for NodeSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "pub struct {} {{", self.name)?;
        for property in &self.properties {
            writeln!(f, "    pub {}: {},", property.name, property.field_type)?;
        }
        writeln!(f, "}}")
    }
}
impl ToTypeScript for NodeSchema {
    fn to_typescript(&self) -> String {
        let mut result = format!("interface {} {{\n", self.name);
        result.push_str("  id: string;\n");

        for property in &self.properties {
            result.push_str(&format!(
                "  {}: {};\n",
                property.name,
                match &property.field_type {
                    GeneratedType::RustType(t) => t.to_ts(),
                    _ => unreachable!(),
                }
            ));
        }

        result.push_str("}\n");
        result
    }
}

#[derive(Clone)]
pub struct EdgeSchema {
    pub name: String,
    pub from: String,
    pub to: String,
    pub properties: Vec<SchemaProperty>,
}
impl Display for EdgeSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "pub struct {} {{", self.name)?;
        writeln!(f, "    pub from: {},", self.from)?;
        writeln!(f, "    pub to: {},", self.to)?;
        for property in &self.properties {
            writeln!(f, "    pub {}: {},", property.name, property.field_type)?;
        }
        writeln!(f, "}}")
    }
}
impl ToTypeScript for VectorSchema {
    fn to_typescript(&self) -> String {
        let mut result = format!("interface {} {{\n", self.name);
        result.push_str("  id: string;\n");
        result.push_str("  data: Array<number>;\n");

        for property in &self.properties {
            result.push_str(&format!(
                "  {}: {};\n",
                property.name,
                match &property.field_type {
                    GeneratedType::RustType(t) => t.to_ts(),
                    _ => unreachable!(),
                }
            ));
        }

        result.push_str("}\n");
        result
    }
}
#[derive(Clone)]
pub struct VectorSchema {
    pub name: String,
    pub properties: Vec<SchemaProperty>,
}
impl Display for VectorSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "pub struct {} {{", self.name)?;
        for property in &self.properties {
            writeln!(f, "    pub {}: {},", property.name, property.field_type)?;
        }
        writeln!(f, "}}")
    }
}
impl ToTypeScript for EdgeSchema {
    fn to_typescript(&self) -> String {
        let properties_str = self
            .properties
            .iter()
            .map(|p| {
                format!(
                    "    {}: {}",
                    p.name,
                    match &p.field_type {
                        GeneratedType::RustType(t) => t.to_ts(),
                        _ => unreachable!(),
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(";");

        format!(
            "interface {} {{\n  id: string;\n  from: {};\n  to: {};\n  properties: {{\n\t{}\n}};\n}}\n",
            self.name, self.from, self.to, properties_str
        )
    }
}

#[derive(Clone)]
pub struct SchemaProperty {
    pub name: String,
    pub field_type: GeneratedType,
    pub default_value: Option<GeneratedValue>,
    // pub is_optional: bool,
    pub is_index: FieldPrefix,
}

pub struct Query {
    pub mcp_handler: Option<String>,
    pub name: String,
    pub statements: Vec<Statement>,
    pub parameters: Vec<Parameter>, // iterate through and print each one
    pub sub_parameters: Vec<(String, Vec<Parameter>)>,
    pub return_values: Vec<ReturnValue>,
    pub is_mut: bool,
}
impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // prints sub parameter structs (e.g. (docs: {doc: String, id: String}))
        for (name, parameters) in &self.sub_parameters {
            writeln!(f, "#[derive(Serialize, Deserialize)]")?;
            writeln!(f, "pub struct {name} {{")?;
            for parameter in parameters {
                writeln!(f, "    pub {}: {},", parameter.name, parameter.field_type)?;
            }
            writeln!(f, "}}")?;
        }
        // prints top level parameters (e.g. (docs: {doc: String, id: String}))
        // if !self.parameters.is_empty() {
        writeln!(f, "#[derive(Serialize, Deserialize)]")?;
        writeln!(f, "pub struct {}Input {{\n", self.name)?;
        write!(
            f,
            "{}",
            self.parameters
                .iter()
                .map(|p| format!("{p}"))
                .collect::<Vec<_>>()
                .join(",\n")
        )?;
        write!(f, "\n}}\n")?;
        // }

        if let Some(mcp_handler) = &self.mcp_handler {
            writeln!(
                f,
                "#[tool_call({}, {})]",
                mcp_handler,
                match self.is_mut {
                    true => "with_write",
                    false => "with_read",
                }
            )?;
        }
        writeln!(
            f,
            "#[handler({})]",
            match self.is_mut {
                true => "with_write",
                false => "with_read",
            }
        )?; // Handler macro

        // prints the function signature
        writeln!(
            f,
            "pub fn {} (input: &HandlerInput) -> Result<Response, GraphError> {{",
            self.name
        )?;
        writeln!(f, "{{")?;

        // prints each statement
        for statement in &self.statements {
            writeln!(f, "    {statement};")?;
        }

        // commit the transaction
        // writeln!(f, "    txn.commit().unwrap();")?;

        // create the return values
        writeln!(
            f,
            "let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();"
        )?;
        if !self.return_values.is_empty() {
            for return_value in &self.return_values {
                writeln!(f, "    {return_value}")?;
            }
        }

        writeln!(f, "}}")?;
        writeln!(f, "}}")
    }
}
impl Default for Query {
    fn default() -> Self {
        Self {
            mcp_handler: None,
            name: "".to_string(),
            statements: vec![],
            parameters: vec![],
            sub_parameters: vec![],
            return_values: vec![],
            is_mut: false,
        }
    }
}

pub struct Parameter {
    pub name: String,
    pub field_type: GeneratedType,
}
impl Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pub {}: {}", self.name, self.field_type)
    }
}

#[derive(Clone)]
pub enum Statement {
    Assignment(Assignment),
    Drop(Drop),
    Traversal(Traversal),
    ForEach(ForEach),
    Literal(GenRef<String>),
    Identifier(GenRef<String>),
    BoExp(BoExp),
    Empty,
}
impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Assignment(assignment) => write!(f, "{assignment}"),
            Statement::Drop(drop) => write!(f, "{drop}"),
            Statement::Traversal(traversal) => write!(f, "{traversal}"),
            Statement::ForEach(foreach) => write!(f, "{foreach}"),
            Statement::Literal(literal) => write!(f, "{literal}"),
            Statement::Identifier(identifier) => write!(f, "{identifier}"),
            Statement::BoExp(bo) => write!(f, "{bo}"),
            Statement::Empty => write!(f, ""),
        }
    }
}

#[derive(Clone)]
pub enum IdentifierType {
    Primitive,
    Traversal,
    Empty,
}

#[derive(Clone)]
pub struct Assignment {
    pub variable: GenRef<String>,
    pub value: Box<Statement>,
}
impl Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {} = {}", self.variable, *self.value)
    }
}

#[derive(Clone)]
pub struct ForEach {
    pub for_variables: ForVariable,
    pub in_variable: ForLoopInVariable,
    pub statements: Vec<Statement>,
}
impl Display for ForEach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.for_variables {
            ForVariable::ObjectDestructure(variables) => {
                write!(
                    f,
                    "for {}Data {{ {} }} in {}",
                    self.in_variable.inner(),
                    variables
                        .iter()
                        .map(|v| format!("{v}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.in_variable
                )?;
            }
            ForVariable::Identifier(identifier) => {
                write!(f, "for {} in {}", identifier, self.in_variable)?;
            }
            ForVariable::Empty => {
                panic!("For variable is empty");
            }
        }
        writeln!(f, " {{")?;
        for statement in &self.statements {
            writeln!(f, "    {statement};")?;
        }
        writeln!(f, "}}")
    }
}

#[derive(Clone)]
pub enum ForVariable {
    ObjectDestructure(Vec<GenRef<String>>),
    Identifier(GenRef<String>),
    Empty,
}
#[derive(Debug, Clone)]
pub enum ForLoopInVariable {
    Identifier(GenRef<String>),
    Parameter(GenRef<String>),
    Empty,
}
impl ForLoopInVariable {
    pub fn inner(&self) -> String {
        match self {
            ForLoopInVariable::Identifier(identifier) => identifier.to_string(),
            ForLoopInVariable::Parameter(parameter) => parameter.to_string(),
            ForLoopInVariable::Empty => "".to_string(),
        }
    }
}
impl Display for ForLoopInVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForLoopInVariable::Identifier(identifier) => write!(f, "{identifier}"),
            ForLoopInVariable::Parameter(parameter) => write!(f, "&data.{parameter}"),
            ForLoopInVariable::Empty => {
                panic!("For loop in variable is empty");
            }
        }
    }
}
#[derive(Clone)]
pub struct Drop {
    pub expression: Traversal,
}
impl Display for Drop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Drop::<Vec<_>>::drop_traversal(
                {},
                Arc::clone(&db),
                &mut txn,
            )?;",
            self.expression
        )
    }
}


/// Boolean expression is used for a traversal or set of traversals wrapped in AND/OR
/// that resolve to a boolean value
#[derive(Clone)]
pub enum BoExp {
    And(Vec<BoExp>),
    Or(Vec<BoExp>),
    Exists(Traversal),
    Expr(Traversal),
}
impl Display for BoExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoExp::And(traversals) => {
                let tr = traversals
                    .iter()
                    .map(|s| format!("{s}"))
                    .collect::<Vec<_>>();
                write!(f, "{}", tr.join(" && "))
            }
            BoExp::Or(traversals) => {
                let tr = traversals
                    .iter()
                    .map(|s| format!("{s}"))
                    .collect::<Vec<_>>();
                write!(f, "{}", tr.join(" || "))
            }
            BoExp::Exists(traversal) => write!(f, "Exist::exists(&mut {traversal})"),
            BoExp::Expr(traversal) => write!(f, "{traversal}"),
        }
    }
}

pub struct ReturnValue {
    pub value: ReturnValueExpr,
    pub return_type: ReturnType,
}
impl Display for ReturnValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.return_type {
            ReturnType::Literal(name) => {
                writeln!(
                    f,
                    "    return_vals.insert({}.to_string(), ReturnValue::from(Value::from({})));",
                    name, self.value
                )
            }
            ReturnType::NamedLiteral(name) => {
                writeln!(
                    f,
                    "    return_vals.insert({}.to_string(), ReturnValue::from(Value::from({})));",
                    name, self.value
                )
            }
            ReturnType::NamedExpr(name) => {
                writeln!(
                    f,
                    "    return_vals.insert({}.to_string(), ReturnValue::from_traversal_value_array_with_mixin({}.clone(), remapping_vals.borrow_mut()));",
                    name, self.value
                )
            }
            ReturnType::SingleExpr(name) => {
                writeln!(
                    f,
                    "    return_vals.insert({}.to_string(), ReturnValue::from_traversal_value_with_mixin({}.clone(), remapping_vals.borrow_mut()));",
                    name, self.value
                )
            }
            ReturnType::UnnamedExpr => {
                write!(f, "// need to implement unnamed return value\n todo!()")?;
                panic!("Unnamed return value is not supported");
            }
        }
    }
}

impl ReturnValue {
    pub fn get_name(&self) -> String {
        match &self.return_type {
            ReturnType::Literal(name) => name.inner().inner().to_string(),
            ReturnType::NamedLiteral(name) => name.inner().inner().to_string(),
            ReturnType::NamedExpr(name) => name.inner().inner().to_string(),
            ReturnType::SingleExpr(name) => name.inner().inner().to_string(),
            ReturnType::UnnamedExpr => todo!(),
        }
    }

    pub fn new_literal(name: GeneratedValue, value: GeneratedValue) -> Self {
        Self {
            value: ReturnValueExpr::Value(value.clone()),
            return_type: ReturnType::Literal(name),
        }
    }
    pub fn new_named_literal(name: GeneratedValue, value: GeneratedValue) -> Self {
        Self {
            value: ReturnValueExpr::Value(value.clone()),
            return_type: ReturnType::NamedLiteral(name),
        }
    }
    pub fn new_named(name: GeneratedValue, value: ReturnValueExpr) -> Self {
        Self {
            value,
            return_type: ReturnType::NamedExpr(name),
        }
    }
    pub fn new_single_named(name: GeneratedValue, value: ReturnValueExpr) -> Self {
        Self {
            value,
            return_type: ReturnType::SingleExpr(name),
        }
    }
    pub fn new_unnamed(value: ReturnValueExpr) -> Self {
        Self {
            value,
            return_type: ReturnType::UnnamedExpr,
        }
    }
}

#[derive(Clone)]
pub enum ReturnType {
    Literal(GeneratedValue),
    NamedLiteral(GeneratedValue),
    NamedExpr(GeneratedValue),
    SingleExpr(GeneratedValue),
    UnnamedExpr,
}
#[derive(Clone)]
pub enum ReturnValueExpr {
    Traversal(Traversal),
    Identifier(GeneratedValue),
    Value(GeneratedValue),
}
impl Display for ReturnValueExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReturnValueExpr::Traversal(traversal) => write!(f, "{traversal}"),
            ReturnValueExpr::Identifier(identifier) => write!(f, "{identifier}"),
            ReturnValueExpr::Value(value) => write!(f, "{value}"),
        }
    }
}
