import { getAddress, type Address } from "viem";

export const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";

const checksumCache = new Map<string, Address>();

export function checksumAddress(address: string): Address {
  const key = address.toLowerCase();
  const cached = checksumCache.get(key);
  if (cached) {
    return cached;
  }

  const checksummed = getAddress(key as Address);
  checksumCache.set(key, checksummed);
  return checksummed;
}

export function isZeroAddress(address: string) {
  return address === ZERO_ADDRESS;
}
