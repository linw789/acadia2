struct AssembledVertex {
	float2 position;
	float4 color;
}

struct FragmentInput {
	float4 color;
}

struct VertexOutput {
	FragmentInput frag_input: FragmentInput; 
	float4 position: SV_POSITION;
}

[shader("vertex")]
VertexOutput vertex_main(AssembledVertex assembled) {
	VertexOutput output;
	output.frag_input.color = assembled.color;
	output.position = float4(assembled.position, 0.0, 1.0);
	return output;
}

struct Fragment {
	float4 color;
}

[shader("fragment")]
Fragment fragment_main(FragmentInput frag_input: FragmentInput) {
	Fragment fragment;
	fragment.color = float4(1.0, 0.0, 0.0, 1.0); // frag_input.color;
	return fragment;
}

