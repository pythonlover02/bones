    vec3 history = texture(u_history, v_uv).rgb;
    float hist_valid = step(1e-6, dot(history, history));
