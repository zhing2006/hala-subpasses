#include "scene.hlsl"
#include "geometry.hlsl"

struct FragmentOutput {
  [[vk::location(0)]] float4 albedo: SV_Target0;
  [[vk::location(1)]] float4 normal: SV_Target1;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  uint material_index = g_push_constants.material_index;
  Material mtrl = g_materials[material_index];

  if (mtrl.base_color_map_index != INVALID_INDEX) {
    float3 base_color = g_textures[mtrl.base_color_map_index].Sample(g_samplers[mtrl.base_color_map_index], input.uv).xyz;
    output.albedo = float4(base_color, 1.0);
  } else {
    output.albedo = float4(mtrl.base_color, 1.0);
  }

  output.normal = float4(input.normal * 0.5 + 0.5, 1.0);

  return output;
}