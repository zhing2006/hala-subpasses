#include "scene.hlsl"
#include "lighting.hlsl"

[[vk::input_attachment_index(0)]]
[[vk::binding(0, 2)]]
SubpassInput _depth_input;

[[vk::input_attachment_index(1)]]
[[vk::binding(1, 2)]]
SubpassInput _albedo_input;

[[vk::input_attachment_index(2)]]
[[vk::binding(2, 2)]]
SubpassInput _normal_input;

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  const float3 color = _albedo_input.SubpassLoad().rgb;

  output.color = float4(color, 1.0);

  return output;
}
