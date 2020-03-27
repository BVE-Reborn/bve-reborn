struct VertexInput {
    uint vert_id : SV_VertexID;
};

// I don't know HLSL array syntax 100%, this line might be very wrong
float2 positions[3] = {float2(0, -0.5), float2(0.5, 0.5), float2(-0.5, 0.5)};

float4 main(VertexInput input) : SV_Position {
    return float4(positions[input.vert_id], 0, 1);
}
