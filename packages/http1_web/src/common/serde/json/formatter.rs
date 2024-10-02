use std::io::Write;

use super::value::Number;

pub trait Formatter<W: Write> {
    fn write_number<N: Into<Number>>(&mut self, writer: &mut W, value: N) -> std::io::Result<()>;
    fn write_bool(&mut self, writer: &mut W, value: bool) -> std::io::Result<()>;
    fn write_str(&mut self, writer: &mut W, value: &str) -> std::io::Result<()>;
    fn write_null(&mut self, writer: &mut W) -> std::io::Result<()>;

    fn write_object_start(&mut self, writer: &mut W) -> std::io::Result<()>;
    fn write_object_end(&mut self, writer: &mut W) -> std::io::Result<()>;

    fn write_array_start(&mut self, writer: &mut W) -> std::io::Result<()>;
    fn write_array_end(&mut self, writer: &mut W) -> std::io::Result<()>;
}

pub struct PrettyFormatter {
    indent: usize,
}

impl PrettyFormatter {
    fn write_indented<W: Write>(&mut self, w: &mut W, value: &[u8]) -> std::io::Result<()> {
        if self.indent > 0 {
            write!(w, "{:indent$}", " ", indent = self.indent)?;
        }

        w.write(value)?;
        Ok(())
    }
}

impl<F, W> Formatter<W> for &mut F
where
    W: Write,
    F: Formatter<W>,
{
    fn write_number<N: Into<Number>>(&mut self, writer: &mut W, value: N) -> std::io::Result<()> {
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
}

impl<W> Formatter<W> for PrettyFormatter
where
    W: Write,
{
    fn write_number<N: Into<Number>>(&mut self, writer: &mut W, value: N) -> std::io::Result<()> {
        match value.into() {
            Number::Float(f) => {
                let s = f.to_string();
                self.write_indented(writer, s.as_bytes())
            }
            Number::Integer(i) => {
                let s = i.to_string();
                self.write_indented(writer, s.as_bytes())
            }
            Number::UInteger(u) => {
                let s = u.to_string();
                self.write_indented(writer, s.as_bytes())
            }
        }
    }

    fn write_bool(&mut self, writer: &mut W, value: bool) -> std::io::Result<()> {
        if value {
            self.write_indented(writer, b"true")
        } else {
            self.write_indented(writer, b"false")
        }
    }

    fn write_null(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.write_indented(writer, b"null")
    }

    fn write_str(&mut self, writer: &mut W, value: &str) -> std::io::Result<()> {
        let s = format!("\"{value}\"");
        self.write_indented(writer, s.as_bytes())
    }

    fn write_object_start(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.write_indented(writer, b"{\n")?;
        self.indent += 1;
        Ok(())
    }

    fn write_object_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.indent -= 1;
        self.write_indented(writer, b"}")
    }

    fn write_array_start(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.write_indented(writer, b"[\n")
    }

    fn write_array_end(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.write_indented(writer, b"]")
    }
}
