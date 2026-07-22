    {
        vec2 fc = v_uv - HALF2;
        float fr = sqrt(dot(fc, fc));
        float ff = mix(1.0, atan(fr) / max(fr, 0.0001), step(0.0001, fr));
        v_uv = HALF2 + fc * ff;
    }
