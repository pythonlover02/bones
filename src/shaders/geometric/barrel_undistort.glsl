    {
        vec2 bc = v_uv - HALF2;
        v_uv = HALF2 + bc / max(1.0 + 0.2 * dot(bc, bc), 0.0001);
    }
