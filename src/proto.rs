pub enum ParserError {
    InvalidLength(String),
    EmptyCommand,
    OnlyOneByte,
    WrongLength(usize),
}

pub enum Command {
    SimpleString { value: String },
    Array { values: Vec<String> },
}

pub struct Parser;

impl Parser {
    pub fn parse<'a>(command: &'a [u8]) -> Result<Command, ParserError> {
        Self::parse_internal(command)
    }

    fn parse_internal<'a>(command: &'a [u8]) -> Result<Command, ParserError> {
        let message = String::from_utf8_lossy(command).to_lowercase();
        let first_byte = command.get(0).ok_or(ParserError::EmptyCommand)?;
        let message = &message.trim().get(1..).ok_or(ParserError::OnlyOneByte)?;
        let commands = message.split("\r\n");

        match first_byte {
            b'*' => Self::parse_array(commands),
            b'+' => Self::parse_simple_string(commands),
            _ => unimplemented!("error"),
        }
    }

    fn parse_array<'a>(
        mut commands: impl Iterator<Item = &'a str>,
    ) -> Result<Command, ParserError> {
        let mut elements_len: usize = match commands.next() {
            Some(x) => x
                .parse()
                .map_err(|_| ParserError::InvalidLength(x.to_string()))?,
            _ => return Err(ParserError::OnlyOneByte),
        };

        let mut values = vec![];
        while let Some(command) = commands.next() {
            elements_len -= 1;
            values.push(command.to_string());
        }

        if elements_len != 0 {
            return Err(ParserError::WrongLength(elements_len));
        }

        Ok(Command::Array { values })
    }

    fn parse_simple_string<'a>(
        mut commands: impl Iterator<Item = &'a str>,
    ) -> Result<Command, ParserError> {
        Ok(Command::SimpleString {
            value: commands.next().ok_or(ParserError::OnlyOneByte)?.to_string(),
        })
    }
}
