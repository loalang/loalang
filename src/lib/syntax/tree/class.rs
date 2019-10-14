use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Class {
    pub id: Id,
    pub class_keyword: Option<Token>,
    pub symbol: Option<Symbol>,
    pub body: Option<ClassBody>,
    pub period: Option<Token>,
}

impl Class {
    pub fn name(&self) -> String {
        match self.symbol {
            Some(ref s) => s.to_string(),
            None => String::new(),
        }
    }
}

impl Node for Class {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref n) = self.class_keyword {
            first_node = n
        } else if let Some(ref n) = self.symbol {
            first_node = n
        } else if let Some(ref n) = self.body {
            first_node = n
        } else if let Some(ref n) = self.period {
            first_node = n
        } else {
            return None;
        }

        if let Some(ref n) = self.period {
            last_node = n
        } else if let Some(ref n) = self.body {
            last_node = n
        } else if let Some(ref n) = self.symbol {
            last_node = n
        } else if let Some(ref n) = self.class_keyword {
            last_node = n
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.class_keyword);
        push!(children, self.symbol);
        push!(children, self.body);
        push!(children, self.period);

        children
    }
}
