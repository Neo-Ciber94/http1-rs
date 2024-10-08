use std::{fmt::Display, io::Write};

use super::number::Number;

pub trait Formatter<W: Write> {
    fn write_number<N>(&mut self, writer: &mut W, value: N) -> std::io::Result<()>
    where
        N: Into<Number> + Display;
    fn write_bool(&mut self, writer: &mut W, value: bool) -> std::io::Result<()>;
    fn write_str(&mut self, writer: &mut W, value: &str) -> std::io::Result<()>;
    fn write_null(&mut self, writer: &mut W) -> std::io::Result<()>;

    fn write_array_start(&mut self, writer: &mut W) -> std::io::Result<()>;
    fn write_array_end(&mut self, writer: &mut W) -> std::io::Result<()>;
    fn write_array_element_begin(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>;
    fn write_array_element_end(&mut self, writer: &mut W) -> std::io::Result<()>;

    fn write_object_start(&mut self, writer: &mut W) -> std::io::Result<()>;
    fn write_object_end(&mut self, writer: &mut W) -> std::io::Result<()>;
    fn write_object_key_begin(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>;
    fn write_object_key_end(&mut self, writer: &mut W) -> std::io::Result<()>;
    fn write_object_value_begin(&mut self, writer: &mut W) -> std::io::Result<()>;
    fn write_object_value_end(&mut self, writer: &mut W) -> std::io::Result<()>;
}

pub struct CompactFormatter;
impl<W: Write> Formatter<W> for CompactFormatter {
    fn write_number<N>(&mut self, writer: &mut W, value: N) -> std::io::Result<()>
    where
        N: Into<Number> + Display,
    {
        let s = value.to_string();
        writer.write_all(s.as_bytes())
    }

    fn write_bool(&mut self, writer: &mut W, value: bool) -> std::io::Result<()> {
        if value {
            writer.write_all(b"true")
        } else {
            writer.write_all(b"false")
        }
    }

    fn write_null(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"null")
    }

    fn write_str(&mut self, writer: &mut W, value: &str) -> std::io::Result<()> {
        let s = format!("\"{value}\"");
        writer.write_all(s.as_bytes())
    }

    fn write_object_start(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"{")?;
        Ok(())
    }

    fn write_object_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"}")
    }

    fn write_array_start(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"[")
    }

    fn write_array_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"]")
    }

    fn write_array_element_begin(&mut self, writer: &mut W, first: bool) -> std::io::Result<()> {
        if first {
            Ok(())
        } else {
            writer.write_all(b",")
        }
    }

    fn write_array_element_end(&mut self, _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }

    fn write_object_key_begin(&mut self, writer: &mut W, first: bool) -> std::io::Result<()> {
        if first {
            Ok(())
        } else {
            writer.write_all(b",")
        }
    }

    fn write_object_key_end(&mut self, _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }

    fn write_object_value_begin(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b":")
    }

    fn write_object_value_end(&mut self, _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct PrettyFormatter {
    level: usize,
    indent: &'static [u8],
}

impl PrettyFormatter {
    pub fn new() -> Self {
        Self::with_indent(b"  ")
    }

    pub fn with_indent(indent: &'static [u8]) -> Self {
        PrettyFormatter { level: 0, indent }
    }
}

impl Default for PrettyFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyFormatter {
    fn write_indented<W: Write>(&mut self, w: &mut W, value: &[u8]) -> std::io::Result<()> {
        for _ in 0..self.level {
            w.write_all(self.indent)?;
        }

        w.write_all(value)?;
        Ok(())
    }
}

impl<F, W> Formatter<W> for &mut F
where
    W: Write,
    F: Formatter<W>,
{
    fn write_number<N>(&mut self, writer: &mut W, value: N) -> std::io::Result<()>
    where
        N: Into<Number> + Display,
    {
        (**self).write_number(writer, value)
    }

    fn write_bool(&mut self, writer: &mut W, value: bool) -> std::io::Result<()> {
        (**self).write_bool(writer, value)
    }

    fn write_str(&mut self, writer: &mut W, value: &str) -> std::io::Result<()> {
        (**self).write_str(writer, value)
    }

    fn write_null(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_null(writer)
    }

    fn write_object_start(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_object_start(writer)
    }

    fn write_object_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_object_end(writer)
    }

    fn write_array_start(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_array_start(writer)
    }

    fn write_array_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_array_end(writer)
    }

    fn write_array_element_begin(&mut self, writer: &mut W, first: bool) -> std::io::Result<()> {
        (**self).write_array_element_begin(writer, first)
    }

    fn write_array_element_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_array_element_end(writer)
    }

    fn write_object_key_begin(&mut self, writer: &mut W, first: bool) -> std::io::Result<()> {
        (**self).write_object_key_begin(writer, first)
    }

    fn write_object_key_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_object_key_end(writer)
    }

    fn write_object_value_begin(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_object_value_begin(writer)
    }

    fn write_object_value_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        (**self).write_object_value_end(writer)
    }
}

impl<W> Formatter<W> for PrettyFormatter
where
    W: Write,
{
    fn write_number<N>(&mut self, writer: &mut W, value: N) -> std::io::Result<()>
    where
        N: Into<Number> + Display,
    {
        let s = value.to_string();
        writer.write_all(s.as_bytes())
    }

    fn write_bool(&mut self, writer: &mut W, value: bool) -> std::io::Result<()> {
        if value {
            writer.write_all(b"true")
        } else {
            writer.write_all(b"false")
        }
    }

    fn write_null(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"null")
    }

    fn write_str(&mut self, writer: &mut W, value: &str) -> std::io::Result<()> {
        let escaped = value
            .to_string()
            .replace("\"", "\\\"")
            .replace("\\", "\\\\")
            .replace("\n", "\\\n")
            .replace("\r", "\\\r")
            .replace("\t", "\\\t")
            .replace("\x08", "\\b")
            .replace("\x0c", "\\f");

        let s = format!("\"{escaped}\"");
        writer.write_all(s.as_bytes())
    }

    fn write_array_start(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.write_indented(writer, b"[\n")?;
        self.level += 1;
        Ok(())
    }

    fn write_array_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.level -= 1;
        self.write_indented(writer, b"\n]")?;
        Ok(())
    }

    fn write_array_element_begin(&mut self, writer: &mut W, first: bool) -> std::io::Result<()> {
        if !first {
            writer.write_all(b",\n")?;
        }
        self.write_indented(writer, b"")?;
        Ok(())
    }

    fn write_array_element_end(&mut self, _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }

    fn write_object_start(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.write_indented(writer, b"{\n")?;
        self.level += 1;
        Ok(())
    }

    fn write_object_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.level -= 1;
        self.write_indented(writer, b"\n}")
    }

    fn write_object_key_begin(&mut self, writer: &mut W, first: bool) -> std::io::Result<()> {
        if !first {
            writer.write_all(b",\n")?;
        }
        self.write_indented(writer, b"")?;
        Ok(())
    }

    fn write_object_key_end(&mut self, _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }

    fn write_object_value_begin(&mut self, writer: &mut W) -> std::io::Result<()> {
        // self.write_indented(writer, b": ")
        writer.write_all(b": ")
    }

    fn write_object_value_end(&mut self, _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}
