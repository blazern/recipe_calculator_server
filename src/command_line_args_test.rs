use command_line_args;

#[test]
fn can_get_command_line_args() {
    let args = command_line_args::get();
    assert!(args.is_ok() || args.is_err());
}

#[test]
fn can_get_valid_command_line_args() {
    let args = vec!("appname", "/path/to/config");
    let args = command_line_args::parse(args);
    args.unwrap();
}

#[test]
fn cant_get_command_line_args_when_input_args_are_invalid() {
    let args = vec!("appname", "/path/to/config", "some_other_param");
    let args = command_line_args::parse(args);
    assert!(args.is_err());
}