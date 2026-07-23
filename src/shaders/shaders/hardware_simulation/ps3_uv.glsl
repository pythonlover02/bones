    {
        vec2 ps3_res = u_resolution * 0.72;
        v_uv = (floor(v_uv * ps3_res) + 0.5) / ps3_res;
    }
