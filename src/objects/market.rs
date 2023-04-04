use crate::objects::orderbook::OrderbookV1;

use std::collections::HashMap;

pub struct Market {
    name: String,
    books: HashMap<String, OrderbookV1>,
    status: HashMap<String, bool>
}

impl Market {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            books: HashMap::<String, OrderbookV1>::new(),
            status: HashMap::<String, bool>::new()
        }
    }

    pub fn contains(&self, symbol: &str) -> bool {
        if self.books.get(symbol).is_some() {
            return true;
        }

        log::trace!("{}: Cannot find book for symbol {}", self.name, symbol);
        false
    }

    pub fn get_book(&self, symbol: &str) -> Option<&OrderbookV1> {
        if let Some(book) = self.books.get(symbol) {
            return Some(book);
        }

        log::trace!("{}: Cannot find book for symbol {}", self.name, symbol);
        None
    }

    pub fn get_mut_book(&mut self, symbol: &str) -> Option<&mut OrderbookV1> {
        if let Some(book) = self.books.get_mut(symbol) {
            return Some(book);
        }

        log::trace!("{}: Cannot find book for symbol {}", self.name, symbol);
        None
    }

    pub fn set_book(&mut self, new_book: OrderbookV1) {
        self.books.insert(String::from(&new_book.symbol), new_book);
    }

    pub fn clear_book(&mut self, symbol: &str) {
        self.books.remove(symbol);
    }

    pub fn is_pending(&self, symbol: &str) -> bool {
        if let Some(pending) = self.status.get(symbol) {
            return *pending;
        }

        false
    }

    pub fn set_pending(&mut self, symbol: &str) {
        self.status.insert(String::from(symbol), true);
    }

    pub fn clear_pending(&mut self, symbol: &str) {
        self.status.insert(String::from(symbol), false);
    }
}
