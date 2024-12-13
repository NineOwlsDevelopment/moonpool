import { PinataSDK } from "pinata-web3";
import { PublicKey } from "@solana/web3.js";
import fs from "fs";
import path from "path";

const pinata = new PinataSDK({
  pinataJwt: process.env.PINATA_JWT || "",
  pinataGateway: process.env.PINATA_GATEWAY || "",
});

const uploadImageToPinata = async (dropletMint: string) => {
  const imageBuffer = fs.readFileSync(
    path.join(__dirname, `./assets/moonpool_logo.jpg`)
  );

  const imageFile = new File([imageBuffer], dropletMint, {
    type: "image/jpeg",
  });

  const upload = await pinata.upload.file(imageFile);
  return upload.IpfsHash;
};

export const uploadToIPFS = async (
  poolName: string,
  symbol: string,
  dropletMint: string
) => {
  try {
    const image = await uploadImageToPinata(dropletMint);

    const upload = await pinata.upload.json({
      name: poolName,
      symbol: symbol,
      description: `${poolName} Moonpool droplet.`,
      image: `${process.env.PINATA_GATEWAY}/ipfs/${image}`,
    });

    return upload.IpfsHash;
  } catch (error) {
    console.error("Error uploading to IPFS:", error);
    throw error;
  }
};
