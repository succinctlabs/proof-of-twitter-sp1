import { bytesToBigInt, fromHex } from "@zk-email/helpers/dist/binary-format";
import { generateEmailVerifierInputs } from "@zk-email/helpers/dist/input-generators";
import { verifyDKIMSignature } from "@zk-email/helpers/dist/dkim/"
export const STRING_PRESELECTOR = "email was meant for @";

export type ITwitterCircuitInputs = {
  twitterUsernameIndex: string;
  address: string;
  emailHeader: string[];
  emailHeaderLength: string;
  pubkey: string[];
  signature: string[];
  emailBody?: string[] | undefined;
  emailBodyLength?: string | undefined;
  precomputedSHA?: string[] | undefined;
  bodyHashIndex?: string | undefined;
};

export async function generateTwitterVerifierCircuitInputs(
  email: string | Buffer,
  ethereumAddress: string
): Promise<ITwitterCircuitInputs> {
  const emailVerifierInputs = await generateEmailVerifierInputs(email, {
    shaPrecomputeSelector: STRING_PRESELECTOR,
  });
  // const dkimResult = await verifyDKIMSignature(Buffer.from(email));
  // console.log(dkimResult)
  const bodyRemaining = emailVerifierInputs.emailBody!.map((c) => Number(c)); // Char array to Uint8Array
  const selectorBuffer = Buffer.from(STRING_PRESELECTOR);
  const usernameIndex =
    Buffer.from(bodyRemaining).indexOf(selectorBuffer) + selectorBuffer.length;

  const address = bytesToBigInt(fromHex(ethereumAddress)).toString();

  return {
    ...emailVerifierInputs,
    twitterUsernameIndex: usernameIndex.toString(),
    address,
  };
}