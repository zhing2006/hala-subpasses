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

  const float depth = _depth_input.SubpassLoad().x;
  if (depth > 0.0) {
    const float3 color = _albedo_input.SubpassLoad().rgb;
    const float3 normal = _normal_input.SubpassLoad().rgb * 2.0 - 1.0;

    const float4 clip = float4(input.uv * 2.0 - 1.0, depth, 1.0);
    const float4 world_w = mul(i_vp_mtx, clip);
    const float3 pos = world_w.xyz / world_w.w;

    const Light light = lights[0];
    const float3 light_2_surface = light.position - pos;
    const float light_distance_sq = dot(light_2_surface, light_2_surface);
    const float attenuation = 1.0 / light_distance_sq;
    const float3 light_dir = light_2_surface * rsqrt(light_distance_sq);

    const float intensity = max(dot(normal, light_dir) * attenuation, 0.0);
    output.color = float4(intensity * color, 1.0);
  } else {
    output.color = float4(0.0, 0.0, 0.0, 1.0);
  }

  return output;
}
