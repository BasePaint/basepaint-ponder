import { Context, ponder } from "ponder:registry";
import { Balance, TotalBalance } from "ponder:schema";
import { eq } from "ponder";

export type TransferEvent = {
  name: "Transfer";
  args: {
    from: `0x${string}`;
    to: `0x${string}`;
    tokenId?: bigint;
    id?: bigint;
    value?: bigint;
    ids?: readonly bigint[];
    values?: readonly bigint[];
  };
};

export type TransferSingleEvent = {
  name: "TransferSingle";
  args: {
    operator: `0x${string}`;
    from: `0x${string}`;
    to: `0x${string}`;
    id: bigint;
    value: bigint;
  };
};

export type TransferBatchEvent = {
  name: "TransferBatch";
  args: {
    operator: `0x${string}`;
    from: `0x${string}`;
    to: `0x${string}`;
    ids: readonly bigint[];
    values: readonly bigint[];
  };
};

export async function trackBalance(
  contract: string,
  event: TransferSingleEvent | TransferBatchEvent | TransferEvent,
  context: any
) {
  if (event.name === "TransferSingle") {
    const fromId = `${contract}_${event.args.from}_${event.args.id}`;
    const toId = `${contract}_${event.args.to}_${event.args.id}`;
    const value = Number(event.args.value);

    if (BigInt(event.args.from) !== 0n) {
      const fromBalance = await context.db.find(Balance, { id: fromId });
      if (fromBalance) {
        await context.db.update(Balance, { id: fromId }).set({
          value: (fromBalance.value ?? 0) - value,
        });
      }
    }

    if (BigInt(event.args.to) !== 0n) {
      const toBalance = await context.db.find(Balance, { id: toId });
      if (toBalance) {
        await context.db.update(Balance, { id: toId }).set({
          value: (toBalance.value ?? 0) + value,
        });
      }
    }

    if (BigInt(event.args.from) !== 0n) {
      const fromTotalBalance = await context.db.find(TotalBalance, { id: `${contract}_${event.args.from}` });
      if (fromTotalBalance) {
        await context.db.update(TotalBalance, { id: `${contract}_${event.args.from}` }).set({
          value: (fromTotalBalance.value ?? 0) - value,
        });
      }
    }

    if (BigInt(event.args.to) !== 0n) {
      const toTotalBalance = await context.db.find(TotalBalance, { id: `${contract}_${event.args.to}` });
      if (toTotalBalance) {
        await context.db.update(TotalBalance, { id: `${contract}_${event.args.to}` }).set({
          value: (toTotalBalance.value ?? 0) + value,
        });
      }
    }
  } else if (event.name === "TransferBatch") {
    for (let i = 0; i < event.args.ids.length; i++) {
      const id = event.args.ids[i];
      const value = Number(event.args.values[i]);
      const fromId = `${contract}_${event.args.from}_${id}`;
      const toId = `${contract}_${event.args.to}_${id}`;

      if (BigInt(event.args.from) !== 0n) {
        const fromBalance = await context.db.find(Balance, { id: fromId });
        if (fromBalance) {
          await context.db.update(Balance, { id: fromId }).set({
            value: (fromBalance.value ?? 0) - value,
          });
        }
      }

      if (BigInt(event.args.to) !== 0n) {
        const toBalance = await context.db.find(Balance, { id: toId });
        if (toBalance) {
          await context.db.update(Balance, { id: toId }).set({
            value: (toBalance.value ?? 0) + value,
          });
        }
      }

      if (BigInt(event.args.from) !== 0n) {
        const fromTotalBalance = await context.db.find(TotalBalance, { id: `${contract}_${event.args.from}` });
        if (fromTotalBalance) {
          await context.db.update(TotalBalance, { id: `${contract}_${event.args.from}` }).set({
            value: (fromTotalBalance.value ?? 0) - value,
          });
        }
      }

      if (BigInt(event.args.to) !== 0n) {
        const toTotalBalance = await context.db.find(TotalBalance, { id: `${contract}_${event.args.to}` });
        if (toTotalBalance) {
          await context.db.update(TotalBalance, { id: `${contract}_${event.args.to}` }).set({
            value: (toTotalBalance.value ?? 0) + value,
          });
        }
      }
    }
  }
}
