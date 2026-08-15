use btclib::types::Block;

pub fn mine_template(mut block: Block, max_rounds: u64) -> Option<Block> {
    block.mine(max_rounds).then_some(block)
}
