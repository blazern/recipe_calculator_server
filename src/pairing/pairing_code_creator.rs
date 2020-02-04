use super::error::Error;

use rand::Rng;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Mutex;

use crate::db::core::app_user::AppUser;
use crate::db::core::connection::DBConnection;
use crate::db::core::pairing_code_range;
use crate::db::core::taken_pairing_code;
use crate::db::core::taken_pairing_code::TakenPairingCode;
use crate::db::core::transaction;

use crate::utils::now_source::DefaultNowSource;
use crate::utils::now_source::NowSource;

use super::error::ErrorKind::InvalidBoundsError;
use super::error::ErrorKind::OutOfPairingCodes;
use super::error::ErrorKind::PersistentStateCorrupted;
use super::error::ErrorKind::SameNamedFamilyExistsError;

lazy_static! {
    static ref TAKEN_NAMES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

/// See documentation for PairingCodeCreatorImpl
pub trait PairingCodeCreator {
    fn borrow_pairing_code(
        &self,
        user: &AppUser,
        connection: &dyn DBConnection,
    ) -> Result<String, Error>;
}

/// Creator of pairing codes. A pairing code is a number in a range (e.g. [0, 9999]).
///
/// Detailed struct implementation description below.
///
/// Users pairing process (simplified):
/// 1. User1 generates a unique pairing code.
/// 2. User2 types the code of User1 in the client app and sends it to the server.
/// 3. Server sees which user is the owner of the code, and pairs the users.
///
/// The pairing process above creates several restrictions:
/// 1. Each generated pairing code must be unique, no 2 users must have same pairing codes.
/// 2. As a consequence, the code must be generated on the server.
/// 3. Because number of pairing codes is limited to 9999, they must have a short
///    life-time and be returned into a "pairing codes pool" after pairing end.
/// 4. Returned pairing code must be randomized so that they could not be guessed.
///
/// There 2 possible implementations that I could come up with:
/// 1. Have a pool of available pairing codes, when a new user asks for a code, take a random
///    code from the pool, return it to the pool after pairing.
///    This implementation is very un-optimal, because we must have a "pool" of 10000 DB rows for
///    it to work, and those 10000 rows won't be used most of the time.
/// 2. Have a pool of available pairing code ranges (e.g. [0-39, 41-500, 502-9999]), choose a random
///    range from the pool, split it into 2 smaller ranges, stick the splited ranges back into
///    a single one after pairing.
///    This implementation is not as straightforward as the 1st one, but it doesn't force us to
///    have 10000 DB rows unless we have 10000 pairing users.
/// Implementation number 2 is used.
///
/// More detailed description of the implementation (tests rely on it):
///
/// Create a random number (RN1) from 0 to 9999.
/// If a free pairing-code-range wrapping RN1 exists, use it for code generation.
/// (e.g., there's a range 20..30, and RN1==25)
/// Otherwise -
/// Select from db 2 pairing-code-ranges: before and after RN1,
/// (e.g., ranges 5..10 and 40..50 when RN1==25)
/// Randomly choose left or right direction.
/// If chosen direction doesn't have pairing-code-range, choose the other one.
/// If neither of the directions have a pairing-code-range, return out-of-codes error.
///
/// Now we have a selected [S..E] pairing-code-range, choose a
/// random number (RN2) within it.
/// Remove the [S..E] range from db.
/// If RN2 != S, add [S..NR2-1] range to db.
/// If RN2 != E, add [RN2+1..E] range to db.
///
/// Return RN2 as the generated code.
#[derive(Debug)]
pub struct PairingCodeCreatorImpl<NS, RCG>
where
    NS: NowSource,
    RCG: RandCodeGenerator,
{
    family: String,
    codes_range_left: i32,
    codes_range_right: i32,
    code_life_length_secs: i64,
    now_source: NS,
    rand_code_generator: RCG,

    /// NOTE1: This field is !Sync by design, because
    /// we want PairingCodeCreatorImpl to be !Sync and currently
    /// there's no easy way to do it in Rust.
    /// PairingCodeCreatorImpl is NOT thread safe because its operations
    /// on DB require mutual exclusion by their design - single code generation
    /// performs multiple operations on DB, and expect used tables to be in a
    /// complex valid state. It's easy to make a hard-to-find error when threading
    /// and external resource modification are involved together.
    _threading_blocker: PhantomData<Box<dyn Send>>,
}

pub type DefaultPairingCodeCreatorImpl =
    PairingCodeCreatorImpl<DefaultNowSource, DefaultRandCodeGenerator>;

impl<NS, RCG> PairingCodeCreator for PairingCodeCreatorImpl<NS, RCG>
where
    NS: NowSource,
    RCG: RandCodeGenerator,
{
    fn borrow_pairing_code(
        &self,
        user: &AppUser,
        connection: &dyn DBConnection,
    ) -> Result<String, Error> {
        transaction::start(connection, || {
            let res = self.borrow_pairing_code_impl(&user, connection);
            // TODO: report 1nd corruption (the 2nd should be reported on top level). https://trello.com/c/u5HO8ZLK/
            match &res {
                Err(Error(PersistentStateCorrupted(_), _)) => {
                    self.fully_reset_persistent_state(connection)?;
                    self.borrow_pairing_code_impl(&user, connection)
                }
                _ => res,
            }
        })
    }
}

pub fn new(
    family: String,
    codes_range_left: i32,
    codes_range_right: i32,
    code_life_length_secs: i64,
) -> Result<DefaultPairingCodeCreatorImpl, Error> {
    new_extended(
        family,
        codes_range_left,
        codes_range_right,
        code_life_length_secs,
        DefaultNowSource {},
        DefaultRandCodeGenerator {},
    )
}

pub fn new_extended<NS1, RCG1>(
    family: String,
    codes_range_left: i32,
    codes_range_right: i32,
    code_life_length_secs: i64,
    now_source: NS1,
    rand_code_generator: RCG1,
) -> Result<PairingCodeCreatorImpl<NS1, RCG1>, Error>
where
    NS1: NowSource,
    RCG1: RandCodeGenerator,
{
    if codes_range_left < 0 {
        return Err(InvalidBoundsError(format!(
            "Bounds must be <= 0, but are {}, {}",
            codes_range_left, codes_range_right
        ))
        .into());
    }
    if codes_range_right < codes_range_left {
        return Err(InvalidBoundsError(format!(
            "Expected left bound <= right bound, got: {}, {}",
            codes_range_left, codes_range_right
        ))
        .into());
    }
    let mut taken_names = TAKEN_NAMES.lock().expect("Expecting working mutex");
    if taken_names.contains(&family) {
        return Err(SameNamedFamilyExistsError(family).into());
    }
    taken_names.insert(family.clone());
    Ok(PairingCodeCreatorImpl {
        family,
        codes_range_left,
        codes_range_right,
        code_life_length_secs,
        now_source,
        rand_code_generator,
        _threading_blocker: PhantomData {},
    })
}

impl<NS, RCG> Drop for PairingCodeCreatorImpl<NS, RCG>
where
    NS: NowSource,
    RCG: RandCodeGenerator,
{
    fn drop(&mut self) {
        let mut taken_names = TAKEN_NAMES.lock().expect("Expecting working mutex");
        taken_names.remove(&self.family);
    }
}

impl<NS, RCG> PairingCodeCreatorImpl<NS, RCG>
where
    NS: NowSource,
    RCG: RandCodeGenerator,
{
    fn borrow_pairing_code_impl(
        &self,
        user: &AppUser,
        connection: &dyn DBConnection,
    ) -> Result<String, Error> {
        self.validate_time(connection)?;
        self.maybe_init_family(connection)?;
        self.free_old_pairing_codes(connection)?;
        self.validate_free_ranges(connection)?;

        let generated_code = self
            .rand_code_generator
            .gen_code(self.codes_range_left, self.codes_range_right);
        let free_range_wrapping = pairing_code_range::select_first_range_with_value_inside(
            generated_code,
            &self.family,
            connection,
        )?;
        let free_range_left = pairing_code_range::select_first_to_the_left_of(
            generated_code + 1,
            &self.family,
            connection,
        )?;
        let free_range_right = pairing_code_range::select_first_to_the_right_of(
            generated_code - 1,
            &self.family,
            connection,
        )?;

        let mut rng = rand::thread_rng();
        let (preferred_side_range, second_side_range) = if rng.gen_bool(0.5) {
            (free_range_left, free_range_right)
        } else {
            (free_range_right, free_range_left)
        };

        let free_range_chosen = match (free_range_wrapping, preferred_side_range, second_side_range)
        {
            (Some(wrapping), _, _) => wrapping,
            (_, Some(side_range), _) => side_range,
            (_, _, Some(side_range)) => side_range,
            _ => return Err(OutOfPairingCodes {}.into()),
        };

        let generated_code = self
            .rand_code_generator
            .gen_code(free_range_chosen.left(), free_range_chosen.right());
        if generated_code != free_range_chosen.left() {
            let new_range_left = pairing_code_range::new(
                free_range_chosen.left(),
                generated_code - 1,
                self.family.to_owned(),
            );
            pairing_code_range::insert(new_range_left, connection)?;
        }
        if generated_code != free_range_chosen.right() {
            let new_range_right = pairing_code_range::new(
                generated_code + 1,
                free_range_chosen.right(),
                self.family.to_owned(),
            );
            pairing_code_range::insert(new_range_right, connection)?;
        }
        pairing_code_range::delete_by_id(free_range_chosen.id(), connection)?;

        let generated_code = taken_pairing_code::new(
            &user,
            generated_code,
            self.now_source.now_secs()?,
            self.family.to_owned(),
        );
        let generated_code = taken_pairing_code::insert(generated_code, connection)?;

        Ok(self.format_generated_code(generated_code.val()))
    }

    fn validate_time(&self, connection: &dyn DBConnection) -> Result<(), Error> {
        let now = self.now_source.now_secs()?;
        let newer_code =
            taken_pairing_code::select_first_newer_than(now, &self.family, connection)?;
        if newer_code.is_some() {
            return Err(PersistentStateCorrupted(format!(
                "Taken code newer than now exist, time went backwards?
                        Now: {}, code: {:?}",
                now, newer_code
            ))
            .into());
        }
        Ok(())
    }

    fn maybe_init_family(&self, connection: &dyn DBConnection) -> Result<(), Error> {
        let any_taken_code = taken_pairing_code::select_any(&self.family, connection)?;
        let any_free_range = pairing_code_range::select_first_to_the_left_of(
            self.codes_range_right + 1,
            &self.family,
            connection,
        )?;
        if any_taken_code.is_none() && any_free_range.is_none() {
            // Our family is not initialized
            let range = pairing_code_range::new(
                self.codes_range_left,
                self.codes_range_right,
                self.family.to_owned(),
            );
            pairing_code_range::insert(range, connection)?;
        }
        Ok(())
    }

    fn free_old_pairing_codes(&self, connection: &dyn DBConnection) -> Result<(), Error> {
        let now = self.now_source.now_secs()?;
        let last_allowed_time = now - self.code_life_length_secs;
        let freed_codes =
            taken_pairing_code::delete_older_than(last_allowed_time, &self.family, connection)?;
        for code in freed_codes {
            self.return_free_ranges_for_freed_code(&code, connection)?;
        }
        Ok(())
    }

    fn return_free_ranges_for_freed_code(
        &self,
        code: &TakenPairingCode,
        connection: &dyn DBConnection,
    ) -> Result<(), Error> {
        // pairing_code -> PC
        // pairing-code-range (PCR) is a S-E pair, which is a range [S, E]
        // Select PCR with: S <= PCD <= E, if it's not None - report an error, return.
        //
        // Select from db 2 pairing-code-ranges (PCR) before and after pairing-code (PC)
        // If E1 == PC-1 _and_ S2 == PC+1: remove both PCR from db, add PCR S1-E2
        // If E1 == PC-1: remove PCR1 from db, add PCR S1-PC
        // If S2 == PC+1: remove PCR2 from db, add PCR PC-E2
        // Otherwise: add PC-PC PCR to db

        let wrapping_range = pairing_code_range::select_first_range_with_value_inside(
            code.val(),
            &self.family,
            connection,
        )?;
        if let Some(wrapping_range) = wrapping_range {
            return Err(PersistentStateCorrupted(format!(
                "Unexpected wrapping range, code: {:?}, range: {:?}",
                code, wrapping_range
            ))
            .into());
        }
        let range_left =
            pairing_code_range::select_first_to_the_left_of(code.val(), &self.family, connection)?;
        let range_right =
            pairing_code_range::select_first_to_the_right_of(code.val(), &self.family, connection)?;
        if let (Some(range_left), Some(range_right)) = (range_left.as_ref(), range_right.as_ref()) {
            if range_left.right() == code.val() - 1 && code.val() + 1 == range_right.left() {
                pairing_code_range::delete_by_id(range_left.id(), connection)?;
                pairing_code_range::delete_by_id(range_right.id(), connection)?;
                let new_range = pairing_code_range::new(
                    range_left.left(),
                    range_right.right(),
                    self.family.to_owned(),
                );
                pairing_code_range::insert(new_range, connection)?;
                return Ok(());
            }
        }
        if let Some(range_left) = range_left {
            if range_left.right() == code.val() - 1 {
                pairing_code_range::delete_by_id(range_left.id(), connection)?;
                let new_range =
                    pairing_code_range::new(range_left.left(), code.val(), self.family.to_owned());
                pairing_code_range::insert(new_range, connection)?;
                return Ok(());
            }
        }
        if let Some(range_right) = range_right {
            if range_right.left() == code.val() + 1 {
                pairing_code_range::delete_by_id(range_right.id(), connection)?;
                let new_range = pairing_code_range::new(
                    code.val(),
                    range_right.right(),
                    self.family.to_owned(),
                );
                pairing_code_range::insert(new_range, connection)?;
                return Ok(());
            }
        }
        let new_range = pairing_code_range::new(code.val(), code.val(), self.family.to_owned());
        pairing_code_range::insert(new_range, connection)?;
        Ok(())
    }

    fn validate_free_ranges(&self, connection: &dyn DBConnection) -> Result<(), Error> {
        let any_taken_code = taken_pairing_code::select_any(&self.family, connection)?;
        if any_taken_code.is_none() {
            let free_ranges = pairing_code_range::select_family(&self.family, connection)?;
            if free_ranges.len() != 1 {
                return Err(PersistentStateCorrupted(format!(
                    "Expected 1 free range when there are 0 taken codes, got: {:?}",
                    free_ranges
                ))
                .into());
            }
            if free_ranges[0].left() != self.codes_range_left
                || free_ranges[0].right() != self.codes_range_right
            {
                return Err(PersistentStateCorrupted(format!(
                    "Expected the single free range to be equal to bounds, got: {:?}",
                    free_ranges[0]
                ))
                .into());
            }
        }
        Ok(())
    }

    fn format_generated_code(&self, generated_code: i32) -> String {
        let required_digits_count = self.codes_range_right.to_string().len();
        let result_short = generated_code.to_string();
        let needed_zeros_count = required_digits_count - result_short.len();
        "0".repeat(needed_zeros_count) + &result_short
    }

    pub fn fully_reset_persistent_state(&self, connection: &dyn DBConnection) -> Result<(), Error> {
        taken_pairing_code::delete_family(&self.family, connection)?;
        pairing_code_range::delete_family(&self.family, connection)?;
        Ok(())
    }
}

/// Used in tests
pub trait RandCodeGenerator {
    fn gen_code(&self, range_left: i32, range_right: i32) -> i32;
}
#[derive(Debug)]
pub struct DefaultRandCodeGenerator;
impl RandCodeGenerator for DefaultRandCodeGenerator {
    fn gen_code(&self, range_left: i32, range_right: i32) -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(range_left, range_right + 1) // [0, 10) == [0, 9]
    }
}

#[cfg(test)]
#[path = "./pairing_code_creator_test.rs"]
mod pairing_code_creator_test;
