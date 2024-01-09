use axiom_client::{
    axiom_codec::{
        special_values::{
            RECEIPT_ADDRESS_IDX, RECEIPT_DATA_IDX_OFFSET, RECEIPT_LOGS_BLOOM_IDX_OFFSET,
            RECEIPT_LOG_IDX_OFFSET,
        },
        HiLo,
    },
    axiom_eth::halo2_base::{
        gates::{GateChip, GateInstructions},
        AssignedValue, Context,
    },
    subquery::{receipt::ReceiptField, types::AssignedReceiptSubquery},
};

use crate::{Fr, SubqueryCaller};

pub struct Receipt<'a> {
    pub block_number: AssignedValue<Fr>,
    pub tx_idx: AssignedValue<Fr>,
    ctx: &'a mut Context<Fr>,
    caller: SubqueryCaller,
}

pub struct Log<'a> {
    pub block_number: AssignedValue<Fr>,
    pub tx_idx: AssignedValue<Fr>,
    pub field_or_log_idx: AssignedValue<Fr>,
    ctx: &'a mut Context<Fr>,
    caller: SubqueryCaller,
}

pub fn get_receipt(
    ctx: &mut Context<Fr>,
    caller: SubqueryCaller,
    block_number: AssignedValue<Fr>,
    tx_idx: AssignedValue<Fr>,
) -> Receipt {
    Receipt {
        block_number,
        tx_idx,
        ctx,
        caller,
    }
}

impl<'a> Receipt<'a> {
    pub fn call(self, field: ReceiptField) -> HiLo<AssignedValue<Fr>> {
        let field_constant = self.ctx.load_constant(Fr::from(field));
        let mut subquery_caller = self.caller.lock().unwrap();
        let topic = self.ctx.load_constant(Fr::zero());
        let zero_event_schema = self.ctx.load_constants(&[Fr::zero(), Fr::zero()]);
        let event_schema = HiLo::from_hi_lo([zero_event_schema[0], zero_event_schema[1]]);
        let subquery = AssignedReceiptSubquery {
            block_number: self.block_number,
            tx_idx: self.tx_idx,
            field_or_log_idx: field_constant,
            topic_or_data_or_address_idx: topic,
            event_schema,
        };
        subquery_caller.call(self.ctx, subquery)
    }

    pub fn log(self, log_idx: AssignedValue<Fr>) -> Log<'a> {
        let log_offset = self
            .ctx
            .load_constant(Fr::from(RECEIPT_LOG_IDX_OFFSET as u64));
        let gate = GateChip::new();
        let log_idx_with_offset = gate.add(self.ctx, log_idx, log_offset);
        Log {
            block_number: self.block_number,
            tx_idx: self.tx_idx,
            field_or_log_idx: log_idx_with_offset,
            ctx: self.ctx,
            caller: self.caller,
        }
    }

    pub fn logs_bloom(self, logs_bloom_idx: usize) -> HiLo<AssignedValue<Fr>> {
        let mut subquery_caller = self.caller.lock().unwrap();
        if logs_bloom_idx >= 8 {
            panic!("logs_bloom_idx range is [0, 8)");
        }
        let field_idx = logs_bloom_idx + RECEIPT_LOGS_BLOOM_IDX_OFFSET;
        let assigned_field_idx = self.ctx.load_constant(Fr::from(field_idx as u64));
        let topic = self.ctx.load_constant(Fr::zero());
        let zero_event_schema = self.ctx.load_constants(&[Fr::zero(), Fr::zero()]);
        let event_schema = HiLo::from_hi_lo([zero_event_schema[0], zero_event_schema[1]]);
        let subquery = AssignedReceiptSubquery {
            block_number: self.block_number,
            field_or_log_idx: assigned_field_idx,
            tx_idx: self.tx_idx,
            topic_or_data_or_address_idx: topic,
            event_schema,
        };
        subquery_caller.call(self.ctx, subquery)
    }
}

impl<'a> Log<'a> {
    pub fn topic(
        self,
        topic_idx: AssignedValue<Fr>,
        event_schema: Option<HiLo<AssignedValue<Fr>>>,
    ) -> HiLo<AssignedValue<Fr>> {
        let mut subquery_caller = self.caller.lock().unwrap();
        let event_schema = event_schema.unwrap_or_else(|| {
            let zero_event_schema = self.ctx.load_constants(&[Fr::zero(), Fr::zero()]);
            HiLo::from_hi_lo([zero_event_schema[0], zero_event_schema[1]])
        });
        let subquery = AssignedReceiptSubquery {
            block_number: self.block_number,
            tx_idx: self.tx_idx,
            field_or_log_idx: self.field_or_log_idx,
            topic_or_data_or_address_idx: topic_idx,
            event_schema,
        };
        subquery_caller.call(self.ctx, subquery)
    }

    pub fn data(
        self,
        data_idx: AssignedValue<Fr>,
        event_schema: Option<HiLo<AssignedValue<Fr>>>,
    ) -> HiLo<AssignedValue<Fr>> {
        let mut subquery_caller = self.caller.lock().unwrap();
        let event_schema = event_schema.unwrap_or_else(|| {
            let zero_event_schema = self.ctx.load_constants(&[Fr::zero(), Fr::zero()]);
            HiLo::from_hi_lo([zero_event_schema[0], zero_event_schema[1]])
        });
        let data_offset = self
            .ctx
            .load_constant(Fr::from(RECEIPT_DATA_IDX_OFFSET as u64));
        let gate = GateChip::new();
        let data_idx_with_offset = gate.add(self.ctx, data_idx, data_offset);
        let subquery = AssignedReceiptSubquery {
            block_number: self.block_number,
            tx_idx: self.tx_idx,
            field_or_log_idx: self.field_or_log_idx,
            topic_or_data_or_address_idx: data_idx_with_offset,
            event_schema,
        };
        subquery_caller.call(self.ctx, subquery)
    }

    pub fn address(self) -> HiLo<AssignedValue<Fr>> {
        let mut subquery_caller = self.caller.lock().unwrap();
        let topic = self.ctx.load_constant(Fr::from(RECEIPT_ADDRESS_IDX as u64));
        let zero_event_schema = self.ctx.load_constants(&[Fr::zero(), Fr::zero()]);
        let event_schema = HiLo::from_hi_lo([zero_event_schema[0], zero_event_schema[1]]);
        let subquery = AssignedReceiptSubquery {
            block_number: self.block_number,
            tx_idx: self.tx_idx,
            field_or_log_idx: self.field_or_log_idx,
            topic_or_data_or_address_idx: topic,
            event_schema,
        };
        subquery_caller.call(self.ctx, subquery)
    }
}
