use std::io::Write;

use toml::{Table, Value};

#[derive(Clone, Debug)]
pub struct XmlWriter<W: Write> {
    write: W,
    open_tags: Vec<String>,
    indent_amount: usize,
}

pub trait XmlWrite {
    fn write<W: Write>(&self, writer: &mut XmlWriter<W>) -> Result<(), &'static str>;
}

impl<W: Write> XmlWriter<W> {
    const ERR_MESSAGE: &str = "error writing to output";

    pub fn new(write: W, indent_amount: usize) -> Self {
        Self {
            write,
            open_tags: Vec::new(),
            indent_amount,
        }
    }

    fn spaces(&self) -> usize {
        self.open_tags.len() * self.indent_amount
    }

    pub fn open_tag(&mut self, tag: String) -> Result<(), &'static str> {
        writeln!(self.write, "{:>spaces$}<{tag}>", "", spaces = self.spaces())
            .map_err(|_| Self::ERR_MESSAGE)?;
        self.open_tags.push(tag);
        Ok(())
    }

    pub fn open_tag_with_attrib(
        &mut self,
        tag: String,
        attrib_key: &str,
        attrib_value: &str,
    ) -> Result<(), &'static str> {
        writeln!(
            self.write,
            "{:>spaces$}<{tag} {attrib_key}=\"{attrib_value}\">",
            "",
            spaces = self.spaces()
        )
        .map_err(|_| Self::ERR_MESSAGE)?;
        self.open_tags.push(tag);
        Ok(())
    }

    pub fn close_tag(&mut self) -> Result<(), &'static str> {
        let tag = self.open_tags.pop().ok_or("no open tags to close")?;
        writeln!(
            self.write,
            "{:>spaces$}</{tag}>",
            "",
            spaces = self.spaces()
        )
        .map_err(|_| Self::ERR_MESSAGE)
    }

    pub fn close_all(&mut self) -> Result<(), &'static str> {
        while !self.open_tags.is_empty() {
            self.close_tag()?;
        }
        Ok(())
    }

    pub fn element(&mut self, tag: &str, value: &str) -> Result<(), &'static str> {
        writeln!(
            self.write,
            "{:>spaces$}<{tag}>{value}</{tag}>",
            "",
            spaces = self.spaces()
        )
        .map_err(|_| Self::ERR_MESSAGE)
    }

    pub fn write_table(&mut self, table: &Table) -> Result<(), &'static str> {
        for (key, value) in table {
            match value {
                Value::String(s) => self.element(key, s)?,
                Value::Integer(n) => self.element(key, &n.to_string())?,
                Value::Float(n) => self.element(key, &n.to_string())?,
                Value::Boolean(b) => self.element(key, &b.to_string())?,
                Value::Datetime(dt) => self.element(key, &dt.to_string())?,
                Value::Array(_) => return Err("unexpected array in TOML"),
                Value::Table(t) => {
                    self.open_tag(key.to_owned())?;
                    self.write_table(t)?;
                    self.close_tag()?;
                }
            }
        }
        Ok(())
    }
}
