use da_primitives::Header;
use frame_support::traits::GenesisBuild;
use frame_system as system;
#[cfg(features = "testing")]
use nomad_base::testing::*;
use nomad_base::NomadBase;
use primitive_types::{H160, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

use crate::{self as da_bridge, FinalizedBlockNumberToBlockHash};

// type TestXt = sp_runtime::testing::TestXt<Call, SignedExtra>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u32;

// TODO: add proper config once frame executive mocking has been demonstrated
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		UpdaterManager: updater_manager::{Pallet, Call, Storage, Event<T>},
		Home: home::{Pallet, Call, Storage, Event<T>},
		DABridge: da_bridge::{Pallet, Call, Storage, Event<T>},
	}
);

frame_support::parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
	pub static ExistentialDeposit: u64 = 0;
}

impl system::Config for Test {
	type AccountData = ();
	type AccountId = AccountId32;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = BlockNumber;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header<Self::BlockNumber, BlakeTwo256>;
	type HeaderBuilder = frame_system::header_builder::da::HeaderBuilder<Test>;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type Origin = Origin;
	type PalletInfo = PalletInfo;
	type Randomness = frame_system::tests::TestRandomness<Test>;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

impl updater_manager::Config for Test {
	type Event = Event;
}

frame_support::parameter_types! {
	pub const MaxMessageBodyBytes: u32 = 2048;
}

impl home::Config for Test {
	type Event = Event;
	type MaxMessageBodyBytes = MaxMessageBodyBytes;
}

frame_support::parameter_types! {
	pub const DABridgePalletId: H256 = H256::zero();
}

impl da_bridge::Config for Test {
	type DABridgePalletId = DABridgePalletId;
	type Event = Event;
}

pub(crate) struct ExtBuilder {
	updater: H160,
	local_domain: u32,
	committed_root: H256,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			updater: Default::default(),
			local_domain: Default::default(),
			committed_root: Default::default(),
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_base(mut self, base: NomadBase) -> Self {
		self.updater = base.updater;
		self.local_domain = base.local_domain;
		self.committed_root = base.committed_root;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		home::GenesisConfig::<Test> {
			updater: self.updater,
			local_domain: self.local_domain,
			committed_root: self.committed_root,
			_phantom: Default::default(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet base storage can be assimilated");
		updater_manager::GenesisConfig::<Test> {
			updater: self.updater,
			_phantom: Default::default(),
		}
		.assimilate_storage(&mut t)
		.expect("Updater manager storage cannot be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<super::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::DABridge(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

pub(crate) fn fill_block_hash_mapping_up_to_n(n: u8) {
	for i in 0..=n {
		FinalizedBlockNumberToBlockHash::<Test>::insert(i as u32, H256::repeat_byte(i))
	}
}