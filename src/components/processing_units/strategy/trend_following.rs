use crate::objects::{Objects, market::Market};

pub struct TrendFollowing {

}

impl TrendFollowing {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn process(&mut self, obj: &Objects, market: &Market) -> Result<(Option<Objects>, bool), String> {
        match obj {
            Objects::OrderbookV1(_) => (),
            Objects::OrderbookUpdateV1(obj) => {
                if obj.symbol == "BTCUSDT" {
                    if let Some(book) = market.get_book(&obj.symbol) {
                        let spread = book.spread();
                        log::info!("Book {} spread {}", obj.symbol, spread);

                        let weighted_spread = book.weighted_spread(0.1);
                        log::info!("Book {} weighted spread {}", obj.symbol, weighted_spread);

                        let mid = book.mid();
                        log::info!("Book {} mid {}", obj.symbol, mid);

                        let weighted_mid = book.weighted_mid();
                        log::info!("Book {} weighted mid {}", obj.symbol, weighted_mid);
                    }
                }
            },
            _ => ()
        };

        Ok((None, false))
    }
}
