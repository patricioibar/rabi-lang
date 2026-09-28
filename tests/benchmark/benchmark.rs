fn main() {
    let cycles = 1000000;
    let mut count = 0;
    let mut acum = 0.0;
    let mut result_text = String::new();
    let mut status = true;

    let mut i = 0;
    while i < cycles {
        count += 1;
        acum += 0.5;

        if i % 2 == 0 {
            status = true;
        } else {
            status = false;
        }

        if status == true {
            result_text = "par".to_string();
        } else {
            result_text = "impar".to_string();
        }

        i += 1;
    }

    println!("{}", count);
    println!("{}", acum);
    println!("{}", result_text);
    println!("{}", status);
}
