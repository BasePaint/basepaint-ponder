import { ponder } from "@/generated";

ponder.on("BasePaintBrush:Approval", async ({ event, context }) => {
  console.log(event.args);
});

ponder.on("BasePaintBrush:ApprovalForAll", async ({ event, context }) => {
  console.log(event.args);
});
