use std::process::Command;

// syncronous 'heavy' process
pub fn process(id: String, prompt: String) -> bool {
    let mut command = Command::new("conda")
        .arg("run")
        .arg("-n")
        .arg("control")
        .arg("python")
        .arg("C:\\Users\\yasushi\\code\\ControlNet\\depth_cli.py")
        .arg(id)
        .arg(prompt)
        // .arg("test")
        // .arg("Los Angeles weather, Santiago transportation, Lima waterfront")
        .spawn()
        .unwrap();

    let result = command.wait();

    match result {
        Ok(code) => {
            if code.success() {
                println!("success!")
            } else {
                println!("command error!")
            }
        }
        Err(_) => println!("error!"),
    }

    true
}
