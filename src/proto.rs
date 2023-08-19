#[derive(Debug)]
pub enum ParserError {
    InvalidLength(String),
    InvalidCommand,
    EmptyFirstByte,
    InvalidInteger(String),
}

#[derive(Debug)]
pub enum Command {
    SimpleString { value: String },
    BulkString { value: String },
    Array { values: Vec<Command> },
    Integer { value: i32 },
}

pub fn parse(command: &[u8]) -> Result<Command, ParserError> {
    let mut iter = command.iter();

    parse_command(&mut iter)
}

fn parse_command<'a>(commands: &mut impl Iterator<Item = &'a u8>) -> Result<Command, ParserError> {
    let first_byte = commands.next().ok_or(ParserError::EmptyFirstByte)?;
    match first_byte {
        b'+' => parse_simple_string(commands),
        b':' => parse_integer(commands),
        b'*' => parse_array(commands),
        b'$' => parse_bulk_string(commands),
        _ => Err(ParserError::EmptyFirstByte),
    }
}

fn parse_array<'a>(commands: &mut impl Iterator<Item = &'a u8>) -> Result<Command, ParserError> {
    let mut values = vec![];
    let elements_len = get_value(commands)?;
    let mut elements_len = elements_len
        .parse::<usize>()
        .map_err(|_| ParserError::InvalidLength(elements_len))?;

    while elements_len > 0 {
        values.push(parse_command(commands)?);
        elements_len -= 1;
    }
    Ok(Command::Array { values })
}

fn parse_bulk_string<'a>(
    commands: &mut impl Iterator<Item = &'a u8>,
) -> Result<Command, ParserError> {
    let len = get_value(commands)?;
    let value = get_value_with_len(
        commands,
        len.parse().map_err(|_| ParserError::InvalidLength(len))?,
    )?;
    Ok(Command::BulkString { value })
}

fn parse_simple_string<'a>(
    commands: &mut impl Iterator<Item = &'a u8>,
) -> Result<Command, ParserError> {
    let value = get_value(commands)?;
    Ok(Command::SimpleString { value })
}

fn parse_integer<'a>(commands: &mut impl Iterator<Item = &'a u8>) -> Result<Command, ParserError> {
    let value = get_value(commands)?;
    Ok(Command::Integer {
        value: value
            .parse()
            .map_err(|_| ParserError::InvalidInteger(value))?,
    })
}

fn get_value_with_len<'a>(
    commands: &mut impl Iterator<Item = &'a u8>,
    mut len: usize,
) -> Result<String, ParserError> {
    let mut value = String::with_capacity(len);
    while len > 0 {
        match commands.next() {
            Some(command) => {
                value.push(*command as char);
            }
            None => return Err(ParserError::InvalidCommand),
        }
        len -= 1;
    }
    let first = commands.next().ok_or(ParserError::InvalidCommand)?;
    let second = commands.next().ok_or(ParserError::InvalidCommand)?;
    if *first != b'\r' || *second != b'\n' {
        return Err(ParserError::InvalidCommand);
    }
    Ok(value)
}

fn get_value<'a>(commands: &mut impl Iterator<Item = &'a u8>) -> Result<String, ParserError> {
    let mut value = String::default();
    loop {
        match commands.next() {
            Some(b'\r') => {
                commands.next();
                return Ok(value);
            }
            Some(command) => {
                value.push(*command as char);
            }
            None => return Err(ParserError::InvalidCommand),
        }
    }
}
