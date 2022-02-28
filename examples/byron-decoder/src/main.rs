use net2::TcpStreamExt;

use pallas::{
    ledger::primitives::{alonzo::BlockWrapper, byron::Block, Fragment},
    network::{
        miniprotocols::{
            blockfetch::{BatchClient, Observer},
            handshake::n2n::{Client, VersionTable},
            run_agent, Point, MAINNET_MAGIC,
        },
        multiplexer::Multiplexer,
    },
};

use std::net::TcpStream;

#[derive(Debug)]
struct BlockPrinter;

impl Observer for BlockPrinter {
    fn on_block_received(&self, body: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", hex::encode(&body));
        println!("----------");

        let block = BlockWrapper::decode_fragment(body.as_slice()).unwrap();
        println!("{:?}", block);
        println!("===========\n\n");

        Ok(())
    }
}

fn main() {
    env_logger::init();

    let bearer = TcpStream::connect("relays-new.cardano-mainnet.iohk.io:3001").unwrap();
    bearer.set_nodelay(true).unwrap();
    bearer.set_keepalive_ms(Some(30_000u32)).unwrap();

    let mut muxer = Multiplexer::setup(bearer, &[0, 3]).unwrap();

    let mut hs_channel = muxer.use_channel(0);
    let versions = VersionTable::v4_and_above(MAINNET_MAGIC);
    let _last = run_agent(Client::initial(versions), &mut hs_channel).unwrap();

    let range = (
        Point::Specific(
            16233554,
            hex::decode("02707620f06fbf6daf2e56848e1b881df1a2b7d3d7ecd53cffc55151858a1de1")
                .unwrap(),
        ),
        Point::Specific(
            16233737,
            hex::decode("77a63ccd3b2b7a9f83686915884ff9c59aab4e00b12c92df46a904dc59e7b5fd")
                .unwrap(),
        ),
    );

    let mut bf_channel = muxer.use_channel(3);
    let bf = BatchClient::initial(range, BlockPrinter {});
    let bf_last = run_agent(bf, &mut bf_channel);
    println!("{:?}", bf_last);
}
