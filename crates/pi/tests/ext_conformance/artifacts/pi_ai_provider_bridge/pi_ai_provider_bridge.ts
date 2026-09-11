import {
    complete,
    completeSimple,
    getApiProvider,
    getModel,
    getModels,
    streamSimpleOpenAIResponses,
} from "@mariozechner/pi-ai";
import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";

export default function(pi: ExtensionAPI) {
    pi.registerTool({
        name: "pi_ai_provider_bridge",
        description: "Verifies @mariozechner/pi-ai helpers resolve through the host bridge.",
        parameters: {
            type: "object",
            properties: {},
            additionalProperties: false,
        },
        execute: async () => {
            const completion = await complete(
                { id: "mock-model" },
                [{ role: "user", content: "complete" }],
                { maxTokens: 8 },
            );
            const simple = await completeSimple("mock-model", "simple", { maxTokens: 4 });
            const streamChunks: string[] = [];
            for await (const chunk of streamSimpleOpenAIResponses("mock-model", "stream")) {
                streamChunks.push(String(chunk));
            }
            const model = await getModel();
            const provider = await getApiProvider();
            const models = await getModels();
            const firstModel = Array.isArray(models) ? models[0] : undefined;

            return {
                content: [
                    {
                        type: "text",
                        text: [
                            `complete:${completion.text}`,
                            `completeSimple:${simple}`,
                            `streamSimpleOpenAIResponses:${streamChunks.join("")}`,
                            `getModel:${model.provider}/${model.modelId}`,
                            `getApiProvider:${provider}`,
                            `getModels:${firstModel ? firstModel.id : "missing"}`,
                        ].join("\n"),
                    },
                ],
            };
        },
    });
}
