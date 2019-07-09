// Encoding library
use parity_codec::Encode;

// Enables access to store a value in runtime storage
// Imports the `Result` type that is returned from runtime functions
// Imports the `decl_module!` and `decl_storage!` macros
use support::{StorageValue, dispatch::Result, decl_module, decl_storage};

// Traits used for interacting with Substrate's Balances module
// `Currency` gives you access to interact with the on-chain currency
// `WithdrawReason` and `ExistenceRequirement` are enums for balance functions
use support::traits::{Currency, WithdrawReason, ExistenceRequirement};

// These are traits which define behavior around math and hashing
use runtime_primitives::traits::{Zero, Hash, Saturating};

// Enables us to verify an call to our module is signed by a user account
use system::ensure_signed;


pub trait Trait: balances::Trait {}


decl_storage! {

  trait Store for Module<T: Trait> as Demo {
    
    Payment get(payment): Option<T::Balance>; // 暂存的支付数据
    Pot get(pot): T::Balance;  // 奖池数据
    Nonce get(nonce): u64;   // 一次性索引
  }

}


decl_module! {
  	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn set_payment(origin, value: T::Balance) -> Result {
            // 验证交易者的签名
            let _ = ensure_signed(origin)?;
            // 判断上一笔支付的值
            if Self::payment().is_none() {
              // 更新支付的值
              <Payment<T>>::put(value);

              // 初始化奖池
              <Pot<T>>::put(value);
            }
            // Return Ok(()) when everything happens successfully
            Ok(())
        }

      fn play(origin) -> Result {
            // 验证并获得信息发送者的地址
            let sender = ensure_signed(origin)?;

            // 确保支付的存储中有数据
            let payment = Self::payment().ok_or("Must have payment amount set")?;

            // 获取 storage 数据
            let mut nonce = Self::nonce();
            let mut pot = Self::pot();

            // 尝试从信息发送者的账户扣除余额，但确保账户不会被回收
            let _ = <balances::Module<T> as Currency<_>>::withdraw(&sender, payment, WithdrawReason::Reserve, ExistenceRequirement::KeepAlive)?;

            // Generate a random hash between 0-255 using a csRNG algorithm
            // 使用伪随机算法生成 0~255 的随机数
            if (<system::Module<T>>::random_seed(), &sender, nonce)
              .using_encoded(<T as system::Trait>::Hashing::hash)
              .using_encoded(|e| e[0] < 128)
              {
                // 如果用户小于 128 则获胜，获得奖池中的奖励，这里需要确保在之前的扣款逻辑中，账户余额充足不至于被系统回收
                let _ = <balances::Module<T> as Currency<_>>::deposit_into_existing(&sender, pot)
                  .expect("`sender` must exist since a transaction is being made and withdraw will keep alive; qed.");

                // 重置奖池
                pot = Zero::zero();
            }

            // 不论输赢更新奖池
            pot = pot.saturating_add(payment);

            // 更新 nonce 
            nonce = nonce.wrapping_add(1);

            // 更新存储数据
            <Pot<T>>::put(pot);
            <Nonce<T>>::put(nonce);

            // 返回标识
            Ok(())
        }
    }
}