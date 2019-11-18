pub trait Service {
    fn xaddr(&self) -> &String;

    fn username(&self) -> &String;

    fn password(&self) -> &String;
}