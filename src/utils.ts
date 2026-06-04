import { Context, Event } from "ponder:registry";
import { Balance, TotalBalance } from "ponder:schema";
import { checksumAddress, isZeroAddress } from "./address";

export async function trackBalance(contract: string, event: Event, context: Context) {
  const contractId = checksumAddress(contract);

  // Determine event type based on event structure
  if ("tokenId" in event.args && "from" in event.args && "to" in event.args) {
    // Handle ERC721 transfers
    const tokenId = event.args.tokenId;
    const from = checksumAddress(event.args.from);
    const to = checksumAddress(event.args.to);
    const fromId = `${contractId}_${from}_${tokenId}`;
    const toId = `${contractId}_${to}_${tokenId}`;
    const value = 1; // ERC721 tokens have value of 1

    if (!isZeroAddress(from)) {
      await context.db
        .insert(Balance)
        .values({
          id: fromId,
          ownerId: from,
          contract: contractId,
          tokenId,
          value: -value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) - value,
        }));
    }

    if (!isZeroAddress(to)) {
      await context.db
        .insert(Balance)
        .values({
          id: toId,
          ownerId: to,
          contract: contractId,
          tokenId,
          value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) + value,
        }));
    }

    if (!isZeroAddress(from)) {
      await context.db
        .insert(TotalBalance)
        .values({
          id: `${contractId}_${from}`,
          ownerId: from,
          contract: contractId,
          value: -value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) - value,
        }));
    }

    if (!isZeroAddress(to)) {
      await context.db
        .insert(TotalBalance)
        .values({
          id: `${contractId}_${to}`,
          ownerId: to,
          contract: contractId,
          value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) + value,
        }));
    }
  } else if ("id" in event.args && "value" in event.args && "from" in event.args && "to" in event.args) {
    // Handle ERC1155 single transfers
    const from = checksumAddress(event.args.from);
    const to = checksumAddress(event.args.to);
    const fromId = `${contractId}_${from}_${event.args.id}`;
    const toId = `${contractId}_${to}_${event.args.id}`;
    const value = Number(event.args.value);

    if (!isZeroAddress(from)) {
      await context.db
        .insert(Balance)
        .values({
          id: fromId,
          ownerId: from,
          contract: contractId,
          tokenId: event.args.id,
          value: -value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) - value,
        }));
    }

    if (!isZeroAddress(to)) {
      await context.db
        .insert(Balance)
        .values({
          id: toId,
          ownerId: to,
          contract: contractId,
          tokenId: event.args.id,
          value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) + value,
        }));
    }

    if (!isZeroAddress(from)) {
      await context.db
        .insert(TotalBalance)
        .values({
          id: `${contractId}_${from}`,
          ownerId: from,
          contract: contractId,
          value: -value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) - value,
        }));
    }

    if (!isZeroAddress(to)) {
      await context.db
        .insert(TotalBalance)
        .values({
          id: `${contractId}_${to}`,
          ownerId: to,
          contract: contractId,
          value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) + value,
        }));
    }
  } else if ("ids" in event.args && "values" in event.args && "from" in event.args && "to" in event.args) {
    // Handle ERC1155 batch transfers
    const from = checksumAddress(event.args.from);
    const to = checksumAddress(event.args.to);
    const fromIsZero = isZeroAddress(from);
    const toIsZero = isZeroAddress(to);

    for (let i = 0; i < event.args.ids.length; i++) {
      const id = event.args.ids[i]!;
      const value = Number(event.args.values[i]);
      const fromId = `${contractId}_${from}_${id}`;
      const toId = `${contractId}_${to}_${id}`;

      if (!fromIsZero) {
        await context.db
          .insert(Balance)
          .values({
            id: fromId,
            ownerId: from,
            contract: contractId,
            tokenId: id,
            value: -value,
          })
          .onConflictDoUpdate((row) => ({
            value: (row.value ?? 0) - value,
          }));
      }

      if (!toIsZero) {
        await context.db
          .insert(Balance)
          .values({
            id: toId,
            ownerId: to,
            contract: contractId,
            tokenId: id,
            value,
          })
          .onConflictDoUpdate((row) => ({
            value: (row.value ?? 0) + value,
          }));
      }

      if (!fromIsZero) {
        await context.db
          .insert(TotalBalance)
          .values({
            id: `${contractId}_${from}`,
            ownerId: from,
            contract: contractId,
            value: -value,
          })
          .onConflictDoUpdate((row) => ({
            value: (row.value ?? 0) - value,
          }));
      }

      if (!toIsZero) {
        await context.db
          .insert(TotalBalance)
          .values({
            id: `${contractId}_${to}`,
            ownerId: to,
            contract: contractId,
            value,
          })
          .onConflictDoUpdate((row) => ({
            value: (row.value ?? 0) + value,
          }));
      }
    }
  }
}
