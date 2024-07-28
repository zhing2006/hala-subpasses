#include "lighting.hlsl"

struct VertexInput {
  [[vk::location(0)]] uint vertex_id: SV_VertexID;
};

ToFragment main(VertexInput input) {
  ToFragment output = (ToFragment)0;

  const float2 uv = float2((input.vertex_id << 1) & 2, input.vertex_id & 2) * 0.5;
  output.position = float4(uv * 2.0 - 1.0, 0.0, 1.0);
  output.uv = uv;

  return output;
}