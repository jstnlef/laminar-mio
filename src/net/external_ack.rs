/// Third party's ack information.
///
/// So what does this mean?
///
/// Here we store information about the other side (virtual connection).
#[derive(Debug, Default)]
pub struct ExternalAcks {
    /// the last sequence number we have received from the other side.
    last_sequence_num: u16,
    /// We define an "ack bitfield" such that each bit corresponds to acks of the 32 sequence
    /// numbers before "ack". So let’s say "ack" is 100. If the first bit of "ack bitfield" is set,
    /// then the packet also includes an ack for packet 99. If the second bit is set, then packet
    /// 98 is acked. This goes all the way down to the 32nd bit for packet 68.
    ack_field: u32
}

impl ExternalAcks {
    /// Acknowledges a packet
    pub fn ack(&mut self, sequence_num: u16) {
        let pos_diff = sequence_num.wrapping_sub(self.last_sequence_num);
        let neg_diff = self.last_sequence_num.wrapping_sub(sequence_num);

        if pos_diff == 0 {
            return;
        }

        // TODO: Reevaluate this logic. Something doesn't seem right...
        if pos_diff < 32000 {
            if pos_diff <= 32 {
                self.ack_field = ((self.ack_field << 1) | 1) << (pos_diff - 1);
            } else {
                self.ack_field = 0;
            }
            // If the packet is more recent, we update the remote sequence to be equal to the sequence number of the packet.
            self.last_sequence_num = sequence_num;
        } else if neg_diff <= 32 {
            self.ack_field |= 1 << (neg_diff - 1);
        }
    }

    /// Accessor for the last sequence number seen
    pub fn last_acked(&self) -> u16 {
        self.last_sequence_num
    }

    /// Accessor for the ack field
    pub fn ack_field(&self) -> u32 {
        self.ack_field
    }
}

#[cfg(test)]
mod test {
    use super::ExternalAcks;

    #[test]
    fn acking_single_packet() {
        let mut acks = ExternalAcks::default();
        acks.ack(0);

        assert_eq!(acks.last_sequence_num, 0);
        assert_eq!(acks.ack_field, 0);
    }

    #[test]
    fn acking_several_packets() {
        let mut acks = ExternalAcks::default();
        acks.ack(0);
        acks.ack(1);
        acks.ack(2);

        assert_eq!(acks.last_sequence_num, 2);
        assert_eq!(acks.ack_field, 1 | (1 << 1));
    }

    #[test]
    fn acking_several_packets_out_of_order() {
        let mut acks = ExternalAcks::default();
        acks.ack(1);
        acks.ack(0);
        acks.ack(2);

        assert_eq!(acks.last_sequence_num, 2);
        assert_eq!(acks.ack_field, 1 | (1 << 1));
    }

    #[test]
    fn acking_a_nearly_full_set_of_packets() {
        let mut acks = ExternalAcks::default();

        for i in 0..32 {
            acks.ack(i);
        }

        assert_eq!(acks.last_sequence_num, 31);
        assert_eq!(acks.ack_field, !0 >> 1);
    }

    #[test]
    fn acking_a_full_set_of_packets() {
        let mut acks = ExternalAcks::default();

        for i in 0..=32 {
            acks.ack(i);
        }

        assert_eq!(acks.last_sequence_num, 32);
        assert_eq!(acks.ack_field, !0);
    }

    #[test]
    fn acking_to_the_edge_forward() {
        let mut acks = ExternalAcks::default();
        acks.ack(0);
        acks.ack(32);

        assert_eq!(acks.last_sequence_num, 32);
        assert_eq!(acks.ack_field, 1 << 31);
    }

    #[test]
    fn acking_too_far_forward() {
        let mut acks = ExternalAcks::default();
        acks.ack(0);
        acks.ack(1);
        acks.ack(34);

        assert_eq!(acks.last_sequence_num, 34);
        assert_eq!(acks.ack_field, 0);
    }

    #[test]
    fn acking_a_whole_buffer_too_far_forward() {
        let mut acks = ExternalAcks::default();
        acks.ack(0);
        acks.ack(60);

        assert_eq!(acks.last_sequence_num, 60);
        assert_eq!(acks.ack_field, 0);
    }

    #[test]
    fn acking_too_far_backward() {
        let mut acks = ExternalAcks::default();
        acks.ack(33);
        acks.ack(0);

        assert_eq!(acks.last_sequence_num, 33);
        assert_eq!(acks.ack_field, 0);
    }

    #[test]
    fn acking_around_zero() {
        let mut acks = ExternalAcks::default();

        for i in 0..33_u16 {
            acks.ack(i.wrapping_sub(16));
        }
        assert_eq!(acks.last_sequence_num, 16);
        assert_eq!(acks.ack_field, !0);
    }

    #[test]
    fn ignores_old_packets() {
        let mut acks = ExternalAcks::default();
        acks.ack(40);
        acks.ack(0);
        assert_eq!(acks.last_sequence_num, 40);
        assert_eq!(acks.ack_field, 0);
    }

    #[test]
    fn ignores_really_old_packets() {
        let mut acks = ExternalAcks::default();
        acks.ack(30000);
        acks.ack(0);
        assert_eq!(acks.last_sequence_num, 30000);
        assert_eq!(acks.ack_field, 0);
    }

    #[test]
    fn skips_missing_acks_correctly() {
        let mut acks = ExternalAcks::default();
        acks.ack(0);
        acks.ack(1);
        acks.ack(6);
        acks.ack(4);
        assert_eq!(acks.last_sequence_num, 6);
        assert_eq!(
            acks.ack_field,
            0        | // 5 (missing)
                (1 << 1) | // 4 (present)
                (0 << 2) | // 3 (missing)
                (0 << 3) | // 2 (missing)
                (1 << 4) | // 1 (present)
                (1 << 5) // 0 (present)
        );
    }
}
