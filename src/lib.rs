/*!
An implementation of Secure Remote Password (SRP6) authentication protocol.

**NOTE**: Please do only use key length >= 2048 bit in production. You can do so by using [`Srp6_2048`] or [`Srp6_4096`].

## Usage
The usage example start on the server side.
Client side interaction is marked explicit when needed.

### 1. A new user, welcome Bob

```rust
use srp6::*;

// this is happening on the client,
// the password is never send to the server at any time
let new_username: UsernameRef = "Bob";
let user_password: &ClearTextPassword = "secret-password";

let (salt_s, verifier_v) = Srp6_2048::default().generate_new_user_secrets(
    new_username,
    user_password
);

assert_eq!(salt_s.num_bytes(), Srp6_2048::KEY_LEN);
assert_eq!(verifier_v.num_bytes(), Srp6_2048::KEY_LEN);

// The server needs to persist,
// `new_username`, `salt_s` and `verifier_v` in a user database / pw file
```
**NOTE:** the password of the user will not be stored!

**NOTE2:** the salt and verifier will never be the same, they have a random component to it

### 2. A session handshake for Bob
On the server side
- when a user/client connects they would send their [`Username`] first
- with the username the server will lookup their [`Salt`] and [`PasswordVerifier`] from a user database or pw file
- with this data the server would start the authentication with a [`Handshake`] send to the client
- the server would also keep a [`HandshakeProofVerifier`] for this user in order to verify the proof he will get from the client
```rust
use srp6::*;

// the username is sent by the client
let user = mocked::lookup_user_details("Bob");

let srp = Srp6_2048::default();
let (handshake, proof_verifier) = srp.start_handshake(&user);
assert_eq!(handshake.s, user.salt);
assert_eq!(handshake.N, srp.N);
assert_eq!(handshake.g, srp.g);
assert_eq!(handshake.B.num_bytes(), Srp6_2048::KEY_LEN);

// TODO: next step: the client calculates proof

# mod mocked {
#     use srp6::*;
#
#     /// this is a mock
#     /// normally this would come from a user database
#     pub fn lookup_user_details(username: UsernameRef) -> UserDetails {
#         use std::convert::TryInto;
#
#         UserDetails {
#             username: username.to_owned(),
#             salt: "CC927E15A5E5B5F420F26A498F14E98D7DC201DCB4CBF4E8E82320AC092A5C0ADE338D7392F7C23C20DDF08D79E3DF83203759887C779B12C18B840A6AEF40A9FCF4D0103C48A832402B07D882F495BFC66A9D6BAAEADF7FEE5965C8BD89CE09FF4572B73DD44DE610514BE19D58B27E4F57641D093B97834EB1D8EAD5BB2DE61777240566DC00AA906E6E5C674ECE33DAC5887685E5BE3E93322CA426715A9B5EF71DF0790459EA638006DCA52B63B6E49CCD239C7F7F8ED60DEA8A85572FEC53991A339A58C1D35962217B2CE57D63A75CD7CF6DEAECEE050684D34D8B4511778C40F3DBFCCBB22A887BA9EDFA894A4D0B83FEADF919F59776A5E969C3AEF4"
#                 .try_into()
#                 .unwrap(),
#             verifier: "9310C7532A50A7266F5F7D26E93DED88C0600D3CD1B7F16B1B3756D4FBA448E5A7D79E5F516332597E46CB44331B9FACD698D8E821B518A289332165AF8BAD0089421528126432598EE979A83A074141E10A6B625394FB8A3E9FFF0858A89B790895EA23AC75A32B15FAD6EFA5E928762AB3BEA4804E67BC290CAA685DB0A1F138AE7ED8424723302918DBBE454DE10F59039244ACCCD0CABF65923291E29DD4CB189BD718D935FABE31AEEA005BE50410E5FCA68D8F1D163ED7A06C37718B0B06528D08522CB9564F3C915384DF69F69E6FDEDFB59145F8AFB27C54402E0078130FA1C93512653C63FF4CFD772E8A82414C31DA9D3627D63BB56ED482BEF3DF"
#                 .try_into()
#                 .unwrap(),
#         }
#     }
# }
```

### 3. A `Proof` that Bob is Bob
- with the handshake, Bob needs to create [`Proof`] that he is Bob
- this [`Proof`] and his [`PublicKey`] will be sent to the server where it is verified

```rust
use srp6::*;

// this is entered by the user
let username = "Bob";
let bobs_password: &ClearTextPassword = "secret-password";

// this comes from the server
let handshake = mocked::handshake_from_the_server(username);

// the final proof calculation
let (proof, strong_proof_verifier) = handshake
    .calculate_proof(username, bobs_password)
    .unwrap();

// `proof` send this proof to the server
// `strong_proof_verifier` is kept for the final verification

# mod mocked {
#     use srp6::*;
#     type Srp6 = Srp6_2048;
#
#     /// this is a mock, nothing you should do on the client side
#     pub fn handshake_from_the_server(username: UsernameRef) -> Handshake<{ Srp6::KEY_LEN }, { Srp6::SALT_LEN }> {
#         let user = lookup_user_details(username);
#         let (handshake, _) = Srp6::default().start_handshake(&user);
#         handshake
#     }
#
#     /// this is a mock
#     /// normally this would come from a user database
#     pub fn lookup_user_details(username: UsernameRef) -> UserDetails {
#         use std::convert::TryInto;
#
#         UserDetails {
#             username: username.to_owned(),
#             salt: "CC927E15A5E5B5F420F26A498F14E98D7DC201DCB4CBF4E8E82320AC092A5C0ADE338D7392F7C23C20DDF08D79E3DF83203759887C779B12C18B840A6AEF40A9FCF4D0103C48A832402B07D882F495BFC66A9D6BAAEADF7FEE5965C8BD89CE09FF4572B73DD44DE610514BE19D58B27E4F57641D093B97834EB1D8EAD5BB2DE61777240566DC00AA906E6E5C674ECE33DAC5887685E5BE3E93322CA426715A9B5EF71DF0790459EA638006DCA52B63B6E49CCD239C7F7F8ED60DEA8A85572FEC53991A339A58C1D35962217B2CE57D63A75CD7CF6DEAECEE050684D34D8B4511778C40F3DBFCCBB22A887BA9EDFA894A4D0B83FEADF919F59776A5E969C3AEF4"
#                 .try_into()
#                 .unwrap(),
#             verifier: "9310C7532A50A7266F5F7D26E93DED88C0600D3CD1B7F16B1B3756D4FBA448E5A7D79E5F516332597E46CB44331B9FACD698D8E821B518A289332165AF8BAD0089421528126432598EE979A83A074141E10A6B625394FB8A3E9FFF0858A89B790895EA23AC75A32B15FAD6EFA5E928762AB3BEA4804E67BC290CAA685DB0A1F138AE7ED8424723302918DBBE454DE10F59039244ACCCD0CABF65923291E29DD4CB189BD718D935FABE31AEEA005BE50410E5FCA68D8F1D163ED7A06C37718B0B06528D08522CB9564F3C915384DF69F69E6FDEDFB59145F8AFB27C54402E0078130FA1C93512653C63FF4CFD772E8A82414C31DA9D3627D63BB56ED482BEF3DF"
#                 .try_into()
#                 .unwrap(),
#         }
#     }
# }
```

### 4. Verify `Proof` from Bob
- The client sends the proof (`[HandshakeProof`]) to the server
- The server calculates his version of the Proof and compoares if they match
- On Success both parties have calculated a strong proof ([`StrongProof`] M2) and a session key ([`StrongSessionKey`] K)

```rust
// this comes from the server
let username = "Bob";
let proof_verifier = mocked::stored_proof_verifier_from_step_2(username);
let proof_from_bob = mocked::bobs_proof();

let strong_proof = proof_verifier.verify_proof(&proof_from_bob);
assert!(strong_proof.is_ok());
let (strong_proof, session_key_server) = strong_proof.unwrap();

// `strong_proof` is sent back to bob

# mod mocked {
#     use srp6::*;
#
#     /// this is a mock, nothing you should do on the client side
#     pub fn stored_proof_verifier_from_step_2(username: UsernameRef) -> HandshakeProofVerifier {
#         let user = lookup_user_details(username);
#         let (_, mut proof_verifier) = Srp6_4096::default().start_handshake(&user);
#         proof_verifier.server_keys = (
#            "3CFF8E64082EFE7D463CD813976FC51109B721787441712F6A197034968BBDBFF1EF7B3006AF0A18CDF273AF3DC35F5E20E78AC167D4DDF34CB11C82D5DECBB07E199227C7F49B2224EA5067DB6B99BB775BD3EEA638E07D49013985B17DF45905A88CE625A000FD563F1A616C3E5C2CA9D0F916524C7AF4030018B0FFB5A8F1D041372F5797AD99F0AD6174267A5A9728C911DCF6252C2A5264B6C7D765E0C39AA0FD53113E2C6A6F692A8B9BBDDC53E523F665B643CC1F0CFA9CCC73CFE1A0CC401E6FB5DF2A489C8D4360283C268E98EE51EBD2593B8F3FEC707DEFB03BB7A6566B119698806A8559138B4CF4A49DE47FEE69FEC9CDE5A090FC5667FC3536623FD05DA51D00F503C31E6DCB88D4D9BAAC6226C1EB1012986DC067B294F0F11368D031501F46BCB1BFF6361DA552D86A7F9E3D300C7444758FBA7D301332DC5AE0067BB61E1ADA80B6B08687AFC0A9137E18630B5CE00BF5398CACBD6FAD5BCAB64D22D647860A8F32D543155FE8D93B2AC97A458DA4C517ED84D3E4E09F72A35455935DD3EF2B87A2AD7F5CB1506BB80D35370E25D140D547019CE51A67A75333A50973396523E92B7E43D59D4CDB3F2F23F426303EC13EE1C89B721AAB6AC4720B3C0E0A7FF88F029A1A6D2EF95F5DAA389125D8D29B077D319409E1C7032496F2BE69C9AE7BD616453658D39A31280653B02981CA702B2483F46A95AC5F"
#                 .try_into()
#                 .unwrap(),
#            "DBC3BF3141ED95FC6A6855825CE1F0FAF9A7C60EDDEA6613D64DBC5D336A69502FC7D1B98FC60F7A85C55C29714CFD5C64161656111893F0E3BA16619E6D33FD78B07E26E948E4E6B52BD4E305B71E381DC2D31F0798381A1331496B39265089A44206F76F485225218148119D4041C409FDF7FD41C5D18AA9979E7F3E4A2F919763CF2A9703D7D02562C06C8B657883D3967B381050C0A8835B2EEED1F672AE3C2A0F9D0983E4808C2F498597FF605DA3B809B182155DE1F81A2B97DA65ED5704D0A438959F6FC588A70C9AD673873321F59E604FD515ED4676884E9A4CB61443A5FCF0AEDBFA573DC18E82C9E32AF4FF48845A77AAE09036D557C1C84211B80A88CC52AED36FA01D1770C84BE064494B9DC1A658AA6B0AF76ADC4E744DAE5EEAC7E8F39C72FCEF572B4A31470858E1025DE2E8572FB229BEF860667C7287F94F1CED40DE1A13D4F5BC6F3FEA65C8FFDCCE2B81F66CE90D2C29D8A5F499BF38CD1FE9C41CC401CC8FC5A796186616208D28CBDD7DAFB747A45796D950FAA54D27FD18E81AF76C2C3096A68BB8A72E059949159C0BD4F378F70A7A9C89D6946DBAE297A3F26DEA11F9E059FD1C8DEA8445BBE040D3DEFD04D15DA30D42F91BB64632AB8A58BCB4DAC5217AEC264222ADCA747FA9E25E8487D9CA5344F31C661FF4CFCEC3783708268BC30E6DE62470C85FFB8D168733A6BBFA0A107ED4BCE584"
#                 .try_into()
#                 .unwrap(),
#         );
#         proof_verifier
#     }
#
#     pub fn bobs_proof() -> HandshakeProof<{Srp6_4096::KEY_LEN}, {Srp6_4096::SALT_LEN}> {
#         HandshakeProof {
#             A: "A23B91C8CEF5EA2311B75BF1F45A6A0AD8EF50264B4D75F5287B3B5635FBDF62AAF4E189AB1942109CCB4AEEA5C233675A81B97A6164DE2980CC6E73A622AE244186BEDCD49446C2646C3E8BECA5361BA485E6EBDB4184CADD6095370E828C40D84295891DA467E67E7B78BBF59AB2F5F546BE9EBDCB65D5D1C9DB498B6C8290F94E71EEB971F1BA3E7D5A2CEEBEC44D1BD4AF54749A6B53967A6C48F6F41612141BE0F35A582F1B0BDEAA3D87BC212C97A98E0DC2CEA350F3D96DF70213EEF7431CA36495F225832F5B992081C9AA0069DA74F821BECD2A56F31C7800464F9195E8545619C332FA7427CA8F27C2967217F4229E75E772CF9DC176E5751731BF5C99998181206F79D8C54E48B82C380275E8F594AB6A5D2FCCD5755C20D29ED22130633E6C1917C43F464D7DF6C171BB769D0228A8B5712E1C849D508875EA96DFF72F47045D4A9F6B86B4102B1225AFEA52A8E9089293291500865DCCF4B7DC3A4A4E171C0822D9EE48A6638E6EDC793BDB94282892DD20AC9ADDD53C24CF8AB5BFAA57144750582E1DBDDE126BFAF4ABD697A0E52B6ED0ED3BEE9DCA4431F7D21FBDAA94D176A481FA2FB6E36527442F45FFFD13701099FA28134F965C81711073719874015F5FB766213B8023DE4774CB020D317B171B8EDD67A85770BCFAEA5FF587E8CBEAAB4A246A95E7A6B18E49CB15EDD8D3D8FB77290CEED83F8EDD"
#                 .try_into()
#                 .unwrap(),
#             M1: "B2FA20E7AC9EEE2F015A4893BA2502B905E452FB"
#                 .try_into()
#                 .unwrap(),
#         }
#     }
#
#     /// this is a mock
#     /// normally this would come from a user database
#     pub fn lookup_user_details(username: UsernameRef) -> UserDetails {
#         use std::convert::TryInto;
#
#         UserDetails {
#             username: username.to_owned(),
#             salt: "2D0F37A1CDFF02593DC50E66BF545EEC9D4E2022EC9974254E5EF452C3606B6FC701EB4C25838B2F4BA40B5A1E91B2FF7DB1224F2BA514E8DEE335C0692CF82E65CC95E546641C41DCC86F69F6293A9E6E52F5A98084700C687110A5658BEC9EC6291C34A2C484869EE8B163AFA3E807DA0D00E9549D5677D3EE530C9CE4401A2E30AF9838838331CEA3500C7E709E38470EA815BDAD7160AD7FA41F6258A29419D9868DD310A6C42B9127463AF5F774437B381C736EDA096BEEE814F1681CBF05EBFA80957AD149F2BD30F23497EECEAFB48973CE76EA15E272AA0C0F9B5C4D905233DC0D080AE28356A5E98C1862D01F3123D6E3BC940A48DAF33CAE6405279878A16F657BBDFCDEC3782C07EEE2638F412A42CC4FD17A13AE1F473DFD4C85A80602165EC1D44D0D3D1CCB0032344A46D84DAC14EB8EF53E4F4DE98CEC70CDAAEBB88601CEEF37D5804BFE273437EFA737AB922BE46D82839D52F274C9C4D33FA849FED7249BC476081782D25A96C658159D8FFC7F97A9D15523FA77171801A26B26955F6E8F54F3BAC6830965C9E7A139ED8AFE8645031A89C283E3ABC73DFD97694FA3F5BB9677DA1B080811B966EE04A2E6E6747EE3567103CB6A7E0B0E2022B93C16F64B8E8FF56DC977FE3D6FA009CA2DA4C250F92607DA0C94A80D475449C30CE39C95890819629F7CDE591446306C3520B75C76152F3134D1BC5828"
#                 .try_into()
#                 .unwrap(),
#             verifier: "8AC4A84B6AC0E46596F60E55372CDC0539F86A464587EEC35B2F0659745FB8BEE5A1073294A899B4FF6B044CE3F5162D7032D8D221545859ECCF7F32AA374FA27E90732E76DEDE4709EA3A5BDC8E6585AB66D26858747A8D95005D6CA51C0FDCEC3B7A864AB296094CC1625FF18CD1A3CF6B4BE3E6105326BC5237E9834B98D75558E48431FF775D72194829DDA86C4169A4297E0977051E6C831F51BBF49C1D429D8485530BEC82D1A475356585E574B42FAA4D426CC37F34B00B4165C1A6EF0EE38DDD561382DD11C29E35D7C0B5B0969982D2A2B682C6E6179004F7ECD29E3B80B22E6207E775C14811124A1FF03FC344398EC4A07BFF03EF45E4D9DB89298A876FAEAECB8D24DAD418C95DB2DB2419FE81B485DA3CD9D8E7D3BED683169A786D149028A4ED2A181C71267CB9B4C20338994B81258533F744E58D21448A0A0218151BAA9F286D36AE9396636DD854D86EDEF55F1AAD9B0B1AD52151B73E44E73E49635C6027C951A0D109D606B9B0182C8D26542C411C215ED9EEE4D5A1B473B879D47EFC951B7A5E8D37177CD0F6A78F08D4DB0F1EC5DD63CA011E03C94284C3A29B656F79B8A26BD70A8FCBBC74789D12677D082EAA2BBCDAFB48A46BD70B4FCC5085C7007FF507D76EC10125CEAACAD0312B42E46FFA7AE1CEFC2E284A49B9EC300EED60FA43C1C9BEE2C688D4FD84A53E69731873D48B97E62AC1EEE1"
#                 .try_into()
#                 .unwrap(),
#         }
#     }
# }
```

### 5. Bob verifies the host
- The client receivs the strong proof ([`StrongProof`] K) from the server
- Bob calculates his own strong proof and verifies the both match
- On Success both parties have verified each other and have a shared strong proof ([`StrongProof`] M2) and a session key ([`StrongSessionKey`] K)

```rust
use srp6::*;

let strong_proof_verifier = mocked::strong_proof_verifier_from_step_3();
let strong_proof = mocked::strong_proof_from_step_4_from_the_server();

let res = strong_proof_verifier.verify_strong_proof(&strong_proof);
assert!(res.is_ok());

# mod mocked {
#    use srp6::*;
#
#    pub fn strong_proof_verifier_from_step_3() -> StrongProofVerifier<{Srp6_4096::KEY_LEN}> {
#         StrongProofVerifier {
#             A: "A23B91C8CEF5EA2311B75BF1F45A6A0AD8EF50264B4D75F5287B3B5635FBDF62AAF4E189AB1942109CCB4AEEA5C233675A81B97A6164DE2980CC6E73A622AE244186BEDCD49446C2646C3E8BECA5361BA485E6EBDB4184CADD6095370E828C40D84295891DA467E67E7B78BBF59AB2F5F546BE9EBDCB65D5D1C9DB498B6C8290F94E71EEB971F1BA3E7D5A2CEEBEC44D1BD4AF54749A6B53967A6C48F6F41612141BE0F35A582F1B0BDEAA3D87BC212C97A98E0DC2CEA350F3D96DF70213EEF7431CA36495F225832F5B992081C9AA0069DA74F821BECD2A56F31C7800464F9195E8545619C332FA7427CA8F27C2967217F4229E75E772CF9DC176E5751731BF5C99998181206F79D8C54E48B82C380275E8F594AB6A5D2FCCD5755C20D29ED22130633E6C1917C43F464D7DF6C171BB769D0228A8B5712E1C849D508875EA96DFF72F47045D4A9F6B86B4102B1225AFEA52A8E9089293291500865DCCF4B7DC3A4A4E171C0822D9EE48A6638E6EDC793BDB94282892DD20AC9ADDD53C24CF8AB5BFAA57144750582E1DBDDE126BFAF4ABD697A0E52B6ED0ED3BEE9DCA4431F7D21FBDAA94D176A481FA2FB6E36527442F45FFFD13701099FA28134F965C81711073719874015F5FB766213B8023DE4774CB020D317B171B8EDD67A85770BCFAEA5FF587E8CBEAAB4A246A95E7A6B18E49CB15EDD8D3D8FB77290CEED83F8EDD"
#                 .try_into()
#                 .unwrap(),
#             M1: "B2FA20E7AC9EEE2F015A4893BA2502B905E452FB"
#                 .try_into()
#                 .unwrap(),
#             K: "BB204D3F39A8D0331A0D9042BFA577D10F6C061CA8ED64FE31C7E6C0E66E3F57BF7994A174CE3EA2"
#                 .try_into()
#                 .unwrap(),
#        }
#    }
#
#    pub fn strong_proof_from_step_4_from_the_server() -> StrongProof {
#        "949384F2F72BFFD3F9B7C0E0E1BC2A6FBEB4C602"
#            .try_into()
#            .unwrap()
#    }
# }
```

## Note on key length
this crate provides some default keys [preconfigured and aliased][defaults].
The modulus prime and genrator numbers are taken from [RFC5054].

## Further details and domain vocabolary
- You can find the documentation of SRP6 [variables in a dedicated module][`protocol_details`].
- [RFC2945](https://datatracker.ietf.org/doc/html/rfc2945) that describes in detail the Secure remote password protocol (SRP).
- [RFC5054] that describes SRP6 for TLS Authentication
- [check out the 2 examples](./examples) that illustrates the srp authentication flow as well

[RFC5054]: (https://datatracker.ietf.org/doc/html/rfc5054)
*/
use thiserror::Error;

// public exports
// pub mod defaults;
// pub mod protocol_details;

// internally available
pub(crate) mod primitives;

mod api;
mod big_number;
mod hash;

pub use api::{new_host::*, get_constants, new_user::*};
// pub use api::user::*;
// pub use defaults::*;
pub use primitives::{
    ClearTextPassword, Generator, MultiplierParameter, PasswordVerifier, PrimeModulus, PrivateKey,
    Proof, PublicKey, Salt, SessionKey, StrongProof, StrongSessionKey, UserCredentials,
    UserDetails, Username, UsernameRef,
};
pub use std::convert::TryInto;

/// encapsulates a [`Srp6Error`]
pub type Result<T> = std::result::Result<T, Srp6Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Srp6Error {
    #[error(
        "The provided key length ({given:?} byte) does not match the expected ({expected:?} byte)"
    )]
    KeyLengthMismatch { given: usize, expected: usize },

    #[error("The provided proof is invalid")]
    InvalidProof(Proof),

    #[error("The provided strong proof is invalid")]
    InvalidStrongProof(StrongProof),

    #[error("The provided public key is invalid")]
    InvalidPublicKey(PublicKey),
}
