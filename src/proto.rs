pub enum ParserError {
    InvalidLength(String),
    EmptyCommand,
    OnlyOneByte,
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
            b'*' => Ok(Self::parse_array(commands)?),
            _ => unimplemented!("error"),
        }
    }

    fn parse_array<'a>(
        mut commands: impl Iterator<Item = &'a str>,
    ) -> Result<Command, ParserError> {
        let elements_len: usize = match commands.next() {
            Some(x) => x
                .parse()
                .map_err(|_| ParserError::InvalidLength(x.to_string()))?,
            _ => 0,
        };

        Ok(Command::Array { values: vec![] })
    }
}
