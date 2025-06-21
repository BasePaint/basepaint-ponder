import { Context, Event } from "ponder:registry";
import { Balance, TotalBalance } from "ponder:schema";

export async function trackBalance(contract: string, event: Event, context: Context) {
  // Determine event type based on event structure
  if ("tokenId" in event.args && "from" in event.args && "to" in event.args) {
    // Handle ERC721 transfers
    const tokenId = event.args.tokenId;
    const fromId = `${contract}_${event.args.from}_${tokenId}`;
    const toId = `${contract}_${event.args.to}_${tokenId}`;
    const value = 1; // ERC721 tokens have value of 1

    if (BigInt(event.args.from) !== 0n) {
      await context.db
        .insert(Balance)
        .values({
          id: fromId,
          ownerId: event.args.from,
          contract,
          tokenId,
          value: -value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) - value,
        }));
    }

    if (BigInt(event.args.to) !== 0n) {
      await context.db
        .insert(Balance)
        .values({
          id: toId,
          ownerId: event.args.to,
          contract,
          tokenId,
          value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) + value,
        }));
    }

    if (BigInt(event.args.from) !== 0n) {
      await context.db
        .insert(TotalBalance)
        .values({
          id: `${contract}_${event.args.from}`,
          ownerId: event.args.from,
          contract,
          value: -value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) - value,
        }));
    }

    if (BigInt(event.args.to) !== 0n) {
      await context.db
        .insert(TotalBalance)
        .values({
          id: `${contract}_${event.args.to}`,
          ownerId: event.args.to,
          contract,
          value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) + value,
        }));
    }
  } else if ("id" in event.args && "value" in event.args && "from" in event.args && "to" in event.args) {
    // Handle ERC1155 single transfers
    const fromId = `${contract}_${event.args.from}_${event.args.id}`;
    const toId = `${contract}_${event.args.to}_${event.args.id}`;
    const value = Number(event.args.value);

    if (BigInt(event.args.from) !== 0n) {
      await context.db
        .insert(Balance)
        .values({
          id: fromId,
          ownerId: event.args.from,
          contract,
          tokenId: event.args.id,
          value: -value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) - value,
        }));
    }

    if (BigInt(event.args.to) !== 0n) {
      await context.db
        .insert(Balance)
        .values({
          id: toId,
          ownerId: event.args.to,
          contract,
          tokenId: event.args.id,
          value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) + value,
        }));
    }

    if (BigInt(event.args.from) !== 0n) {
      await context.db
        .insert(TotalBalance)
        .values({
          id: `${contract}_${event.args.from}`,
          ownerId: event.args.from,
          contract,
          value: -value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) - value,
        }));
    }

    if (BigInt(event.args.to) !== 0n) {
      await context.db
        .insert(TotalBalance)
        .values({
          id: `${contract}_${event.args.to}`,
          ownerId: event.args.to,
          contract,
          value,
        })
        .onConflictDoUpdate((row) => ({
          value: (row.value ?? 0) + value,
        }));
    }
  } else if ("ids" in event.args && "values" in event.args && "from" in event.args && "to" in event.args) {
    // Handle ERC1155 batch transfers
    for (let i = 0; i < event.args.ids.length; i++) {
      const id = event.args.ids[i]!;
      const value = Number(event.args.values[i]);
      const fromId = `${contract}_${event.args.from}_${id}`;
      const toId = `${contract}_${event.args.to}_${id}`;

      if (BigInt(event.args.from) !== 0n) {
        await context.db
          .insert(Balance)
          .values({
            id: fromId,
            ownerId: event.args.from,
            contract,
            tokenId: id,
            value: -value,
          })
          .onConflictDoUpdate((row) => ({
            value: (row.value ?? 0) - value,
          }));
      }

      if (BigInt(event.args.to) !== 0n) {
        await context.db
          .insert(Balance)
          .values({
            id: toId,
            ownerId: event.args.to,
            contract,
            tokenId: id,
            value,
          })
          .onConflictDoUpdate((row) => ({
            value: (row.value ?? 0) + value,
          }));
      }

      if (BigInt(event.args.from) !== 0n) {
        await context.db
          .insert(TotalBalance)
          .values({
            id: `${contract}_${event.args.from}`,
            ownerId: event.args.from,
            contract,
            value: -value,
          })
          .onConflictDoUpdate((row) => ({
            value: (row.value ?? 0) - value,
          }));
      }

      if (BigInt(event.args.to) !== 0n) {
        await context.db
          .insert(TotalBalance)
          .values({
            id: `${contract}_${event.args.to}`,
            ownerId: event.args.to,
            contract,
            value,
          })
          .onConflictDoUpdate((row) => ({
            value: (row.value ?? 0) + value,
          }));
      }
    }
  }
}
