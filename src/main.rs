use micro;

#[derive(micro::Micro)]
pub(crate) struct MyStruct<'a> {
    name: &'a str,
    age: u32,
    hobbies: Vec<String>
}

fn main() {
    println!("Hello World");
}
