    {
        vec2 rd = (HALF2 - v_uv) * 0.2;
        vec3 rs = c
            + texture(u_input, v_uv + rd * 0.14285714).rgb
            + texture(u_input, v_uv + rd * 0.28571429).rgb
            + texture(u_input, v_uv + rd * 0.42857143).rgb
            + texture(u_input, v_uv + rd * 0.57142857).rgb
            + texture(u_input, v_uv + rd * 0.71428571).rgb
            + texture(u_input, v_uv + rd * 0.85714286).rgb
            + texture(u_input, v_uv + rd).rgb;
        c = mix(c, rs * 0.125, 0.5);
    }
