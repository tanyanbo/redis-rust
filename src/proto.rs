pub enum ParserError {
    InvalidLength(String),
    InvalidCommand,
    EmptyFirstByte,
}

pub enum Command {
    SimpleString { value: String },
    BulkString { value: String },
    Array { values: Vec<String> },
}

pub fn parse<'a>(command: &'a [u8]) -> Result<Command, ParserError> {
    let mut iter = command.iter();

    loop {
        let command = parse_command(&mut iter);
    }
}

fn parse_command<'a>(commands: &mut impl Iterator<Item = &'a u8>) -> Result<Command, ParserError> {
    let first_byte = commands.next().ok_or(ParserError::EmptyFirstByte)?;
    match first_byte {
        b'+' => parse_simple_string(commands),
        b'*' => parse_array(commands),
        b'$' => parse_bulk_string(commands),
        _ => Err(ParserError::EmptyFirstByte),
    }
}

fn parse_array<'a>(commands: &mut impl Iterator<Item = &'a u8>) -> Result<Command, ParserError> {
    let mut values = vec![];
    let mut elements_len = String::default();
    while let Some(len) = commands.next() {
        elements_len.push(*len as char);
    }
    let mut elements_len = elements_len
        .parse::<usize>()
        .map_err(|_| ParserError::InvalidLength(elements_len))?;

    while elements_len > 0 {
        elements_len -= 1;

        loop {
            let mut value = String::default();
            match commands.next() {
                Some(command) if *command != b'\r' => {
                    value.push(*command as char);
                }
                Some(_) => {
                    commands.next();
                    break;
                }
                None => return Err(ParserError::InvalidCommand),
            }
        }
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
    loop {
        let mut value = String::default();
        match commands.next() {
            Some(command) if *command != b'\r' => {
                value.push(*command as char);
            }
            Some(_) => {
                commands.next();
                break Ok(value);
            }
            None => return Err(ParserError::InvalidCommand),
        }
    }
}
