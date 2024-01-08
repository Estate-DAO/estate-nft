// import { estate_dao_nft_backend } from "../../declarations/estate_dao_nft_backend";

// import { HttpAgent } from "@dfinity/agent";
// import { AssetManager } from "@dfinity/assets";
// import { Principal } from '@dfinity/principal';
// const { Ed25519KeyIdentity } = require("@dfinity/identity");
// // import fetch from 'isomorphic-fetch';
// // import { Secp256k1KeyIdentity } from "@dfinity/identity-secp256k1";

// const HOST = `http://127.0.0.1:4943`;

// var form = document.getElementById('id2');

// form.addEventListener('change', async (event) => {

//     console.log("2nd st id")
//     const encoder = new TextEncoder();
  
//     const seed = new Uint8Array(32);
//     const base = encoder.encode("test_id2");
//     seed.set(base, 0);
//     seed.fill(2);
//     const testIdentity2 = Ed25519KeyIdentity.generate(seed);
  
//     console.log("2nd identity", testIdentity2.getPrincipal().toText());
  
//     const principal_canister_id = 'jyz3t-veaaa-aaaaa-qabva-cai';
    
//     const agent = new HttpAgent({ host: HOST, identity: testIdentity2});
//     await agent.fetchRootKey();
  
//     const assetManager = new AssetManager({
//       canisterId: principal_canister_id, // Principal of assets canister
//       agent: agent, // Identity in agent must be authorized by the assets canister to make any changes
//     });
//     // console.log("")
//     const file = event.target.files[0];
//     console.log(file);
  
//     try {
//       console.log("new");
//       const key = await assetManager.store(file);
//       console.log('new id Image uploaded successfully');
//       // console.log(key)
//       const files = await assetManager.list();
//       console.log('list assets: ', files);
//     } catch (error) {
//       console.error('Failed to upload image:', error);
//     }
// });


// var get_image = document.getElementById('id3');
// get_image.addEventListener("click", async() => {

//   const encoder = new TextEncoder();
//   const seed = new Uint8Array(32);
//   const base = encoder.encode("test_id2");
//   seed.set(base, 0);
//   seed.fill(2);
//   const testIdentity2 = Ed25519KeyIdentity.generate(seed);

//   console.log(" get funtion using 2nd identity", testIdentity2.getPrincipal().toText());

//   const principal_canister_id = 'jyz3t-veaaa-aaaaa-qabva-cai';
  
//   const agent = new HttpAgent({ host: HOST, identity: testIdentity2});
//   await agent.fetchRootKey();

//   const assetManager = new AssetManager({
//     canisterId: principal_canister_id, // Principal of assets canister
//     agent: agent, // Identity in agent must be authorized by the assets canister to make any changes
//   });

//   const asset = await assetManager.get('/zenitsu.png');
//   const blob = await asset.toBlob();
//   const url = URL.createObjectURL(blob);

//   window.open(URL.createObjectURL(blob, '_blank'));
  
//   // data.write('/zenitsu.png');
//   // console.log("image data:", data);
// })


// var delete_image = document.getElementById('id_delete');
// delete_image.addEventListener("click", async() => {

//   const encoder = new TextEncoder();
//   const seed = new Uint8Array(32);
//   const base = encoder.encode("test_id2");
//   seed.set(base, 0);
//   seed.fill(2);
//   const testIdentity2 = Ed25519KeyIdentity.generate(seed);

//   console.log(" get funtion using 2nd identity", testIdentity2.getPrincipal().toText());

//   const principal_canister_id = 'jyz3t-veaaa-aaaaa-qabva-cai';
  
//   const agent = new HttpAgent({ host: HOST, identity: testIdentity2});
//   await agent.fetchRootKey();

//   const assetManager = new AssetManager({
//     canisterId: principal_canister_id, // Principal of assets canister
//     agent: agent, // Identity in agent must be authorized by the assets canister to make any changes
//   });

//   const del = await assetManager.delete('/zenitsu.png');
//   console.log("imagde deleted succesfully")
//   const files = await assetManager.list();
//   console.log('list assets: ', files);

// })

// document.querySelector("form").addEventListener("change", async (e) => {
 
//   console.log("1st id")
//   const encoder = new TextEncoder();

//   const seed = new Uint8Array(32);
//   const base = encoder.encode("test");
//   seed.set(base, 0);
//   seed.fill(0);
//   const testIdentity = Ed25519KeyIdentity.generate(seed);

//   console.log(testIdentity.getPrincipal().toText());

//   const principal_canister_id = 'jyz3t-veaaa-aaaaa-qabva-cai';
  
//   const agent = new HttpAgent({ host: HOST, identity: testIdentity});
//   await agent.fetchRootKey();

//   const assetManager = new AssetManager({
//     canisterId: principal_canister_id, // Principal of assets canister
//     agent: agent, // Identity in agent must be authorized by the assets canister to make any changes
//   });
//   console.log("0000")
//   // const files = await assetManager.list();
//   // console.log(files)
//   const file = e.target.files[0];
//   console.log(file);

//   try {
//     console.log("111");
//     const key = await assetManager.store(file);
//     console.log('Image uploaded successfully');
//     console.log(key)
//     const files = await assetManager.list();
//     console.log('list assets: ', files);

//   } catch (error) {
//     console.error('Failed to upload image:', error);
//   }
// })


// function create_identity(seed)
// {
//   if(seed == 1){
//     const encoder = new TextEncoder();

//     const seed = new Uint8Array(32);
//     const base = encoder.encode("test_id2");
//     seed.set(base, 0);
//     seed.fill(0);
//     const testIdentity = Ed25519KeyIdentity.generate(seed);
  
//     console.log("1st identity", testIdentity.getPrincipal().toText());
//     return testIdentity;
//   }
//   else {
//     const encoder = new TextEncoder();

//     const seed = new Uint8Array(32);
//     const base = encoder.encode("test_id2");
//     seed.set(base, 0);
//     seed.fill(2);
//     const testIdentity2 = Ed25519KeyIdentity.generate(seed);
  
//     console.log("2nd identity", testIdentity2.getPrincipal().toText());
//     return testIdentity2;
//   }
// }

// var authorize = document.getElementById('id_auth');
// authorize.addEventListener("click", async() => {

//   // console.log("111")
//   // const seed = "another garlic runway license chat model talk double matter glory inner near sauce dune awesome ribbon worth broom similar option pudding fabric float pizza";  

//   // const testIdentity2 = Ed25519KeyIdentity.generate(seed);
//   // const testIdentity2 = Secp256k1KeyIdentity.fromSeedPhrase(seed);

//   // const privateKey = "MHQCAQEEILBeb6z+VxM0faoOhCli/ndg+ItfVSmTn5bXhROVjK1goAcGBSuBBAAKoUQDQgAEi5an8ibdT8IH/pz4bZy8SgAZVVAoQlzhQchL9UChkCZB+Js/a6DM5LqTVd0DBX0HUu593Wok0cQzwiRIy/04JA==";
//   // const secretKeyBuffer = Buffer.from(privateKey, 'hex');
//   // const testIdentity2 = Ed25519KeyIdentity.fromSecretKey(secretKeyBuffer);

//   // const s_key = await fetch('/home/shrey/.config/dfx/identity/default/identity.pem');
//   // const key = buffer.toString('utf-8');
//   // const privateKey = crypto.createHash('sha256').update(key).digest('base64');
//   // const privateKey = "MHQCAQEEILBeb6z+VxM0faoOhCli/ndg+ItfVSmTn5bXhROVjK1goAcGBSuBBAAKoUQDQgAEi5an8ibdT8IH/pz4bZy8SgAZVVAoQlzhQchL9UChkCZB+Js/a6DM5LqTVd0DBX0HUu593Wok0cQzwiRIy/04JA==";
//   // console.log("222");
//   // const base64PrivateKey = btoa(s_key.text());
//   // console.log(base64PrivateKey)
//   // // const privateKey = new Uint8Array(atob(base64PrivateKey));

//   // const id = Ed25519KeyIdentity.fromSecretKey(base64PrivateKey);
//   // console.log("333");

//   // console.log(" local identity", id.getPrincipal().toText());
//   // console.log("444");


//   // const pemData = await fetch("/home/shrey/.config/dfx/identity/default/identity.pem");
//   // const secretKey = identity.Pem.decode(pemData.text());
//   // const identityType = await identity.Identity.create({
//   //   secretKey,
//   // });
//   // console.log(" local identity", identityType.getPrincipal().toText());

//     const encoder = new TextEncoder();

//     const seed = new Uint8Array(32);
//     const base = encoder.encode("test_id2");
//     seed.set(base, 0);
//     seed.fill(2);
//     const testIdentity2 = Ed25519KeyIdentity.generate(seed);
  
//     console.log("2nd identity", testIdentity2.getPrincipal().toText());
//     // return testIdentity2

// })