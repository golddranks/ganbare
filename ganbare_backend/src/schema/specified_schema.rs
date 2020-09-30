table! {
    anon_aliases (id) {
        id -> Int4,
        name -> Varchar,
        user_id -> Nullable<Int4>,
        group_id -> Nullable<Int4>,
    }
}

table! {
    audio_bundles (id) {
        id -> Int4,
        listname -> Varchar,
    }
}

table! {
    audio_files (id) {
        id -> Int4,
        narrators_id -> Int4,
        bundle_id -> Int4,
        file_path -> Varchar,
        mime -> Varchar,
        file_sha2 -> Nullable<Bytea>,
    }
}

table! {
    due_items (id) {
        id -> Int4,
        user_id -> Int4,
        due_date -> Timestamptz,
        due_delay -> Int4,
        cooldown_delay -> Timestamptz,
        correct_streak_overall -> Int4,
        correct_streak_this_time -> Int4,
        item_type -> Varchar,
    }
}

table! {
    e_answered_data (id) {
        id -> Int4,
        answered_date -> Timestamptz,
        active_answer_time_ms -> Int4,
        full_answer_time_ms -> Int4,
        audio_times -> Int4,
        answer_level -> Nullable<Int4>,
        full_spent_time_ms -> Int4,
        reflected_time_ms -> Int4,
    }
}

table! {
    e_asked_data (id) {
        id -> Int4,
        exercise_id -> Int4,
        word_id -> Int4,
    }
}

table! {
    event_experiences (user_id,
    event_id) {
        user_id -> Int4,
        event_id -> Int4,
        event_init -> Timestamptz,
        event_finish -> Nullable<Timestamptz>,
    }
}

table! {
    event_userdata (id) {
        id -> Int4,
        user_id -> Int4,
        event_id -> Int4,
        created -> Timestamptz,
        key -> Nullable<Varchar>,
        data -> Text,
    }
}

table! {
    events (id) {
        id -> Int4,
        name -> Varchar,
        published -> Bool,
        required_group -> Nullable<Int4>,
        priority -> Int4,
    }
}

table! {
    exercise_data (due,
    exercise_id) {
        exercise_id -> Int4,
        due -> Int4,
    }
}

table! {
    exercise_variants (id) {
        id -> Int4,
        exercise_id -> Int4,
    }
}

table! {
    exercises (id) {
        id -> Int4,
        skill_id -> Int4,
        published -> Bool,
        skill_level -> Int4,
    }
}

table! {
    group_memberships (user_id,
    group_id) {
        user_id -> Int4,
        group_id -> Int4,
        anonymous -> Bool,
    }
}

table! {
    narrators (id) {
        id -> Int4,
        name -> Varchar,
        published -> Bool,
    }
}

table! {
    passwords (id) {
        id -> Int4,
        password_hash -> Bytea,
        salt -> Bytea,
        initial_rounds -> Int2,
        extra_rounds -> Int2,
    }
}

table! {
    pending_email_confirms (secret) {
        secret -> Varchar,
        email -> Varchar,
        groups -> Array<Int4>,
        added -> Timestamptz,
    }
}

table! {
    pending_items (id) {
        id -> Int4,
        user_id -> Int4,
        audio_file_id -> Int4,
        asked_date -> Timestamptz,
        pending -> Bool,
        item_type -> Varchar,
        test_item -> Bool,
    }
}

table! {
    q_answered_data (id) {
        id -> Int4,
        answered_qa_id -> Nullable<Int4>,
        answered_date -> Timestamptz,
        active_answer_time_ms -> Int4,
        full_answer_time_ms -> Int4,
        full_spent_time_ms -> Int4,
    }
}

table! {
    q_asked_data (id) {
        id -> Int4,
        question_id -> Int4,
        correct_qa_id -> Int4,
    }
}

table! {
    question_answers (id) {
        id -> Int4,
        question_id -> Int4,
        a_audio_bundle -> Nullable<Int4>,
        q_audio_bundle -> Int4,
        answer_text -> Varchar,
    }
}

table! {
    question_data (due,
    question_id) {
        question_id -> Int4,
        due -> Int4,
    }
}

table! {
    quiz_questions (id) {
        id -> Int4,
        skill_id -> Int4,
        q_name -> Varchar,
        q_explanation -> Varchar,
        question_text -> Varchar,
        published -> Bool,
        skill_level -> Int4,
    }
}

table! {
    reset_email_secrets (user_id) {
        user_id -> Int4,
        email -> Varchar,
        secret -> Varchar,
        added -> Timestamptz,
    }
}

table! {
    sessions (id) {
        id -> Int4,
        user_id -> Int4,
        started -> Timestamptz,
        last_seen -> Timestamptz,
        secret -> Bytea,
        refresh_count -> Int4,
    }
}

table! {
    skill_data (user_id,
    skill_nugget) {
        user_id -> Int4,
        skill_nugget -> Int4,
        skill_level -> Int4,
    }
}

table! {
    skill_nuggets (id) {
        id -> Int4,
        skill_summary -> Varchar,
    }
}

table! {
    user_groups (id) {
        id -> Int4,
        group_name -> Varchar,
        anonymous -> Bool,
    }
}

table! {
    user_metrics (id) {
        id -> Int4,
        new_words_since_break -> Int4,
        new_words_today -> Int4,
        quizes_since_break -> Int4,
        quizes_today -> Int4,
        break_until -> Timestamptz,
        today -> Timestamptz,
        max_words_since_break -> Int4,
        max_words_today -> Int4,
        max_quizes_since_break -> Int4,
        max_quizes_today -> Int4,
        break_length -> Int4,
        delay_multiplier -> Int4,
        initial_delay -> Int4,
        streak_limit -> Int4,
        cooldown_delay -> Int4,
        streak_skill_bump_criteria -> Int4,
    }
}

table! {
    user_stats (id) {
        id -> Int4,
        days_used -> Int4,
        all_active_time_ms -> Int8,
        all_spent_time_ms -> Int8,
        all_words -> Int4,
        quiz_all_times -> Int4,
        quiz_correct_times -> Int4,
        last_nag_email -> Nullable<Timestamptz>,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Nullable<Varchar>,
        joined -> Timestamptz,
        last_seen -> Timestamptz,
    }
}

table! {
    w_answered_data (id) {
        id -> Int4,
        full_spent_time_ms -> Int4,
        audio_times -> Int4,
        checked_date -> Timestamptz,
        active_answer_time_ms -> Int4,
    }
}

table! {
    w_asked_data (id) {
        id -> Int4,
        word_id -> Int4,
        show_accents -> Bool,
    }
}

table! {
    words (id) {
        id -> Int4,
        word -> Varchar,
        explanation -> Varchar,
        audio_bundle -> Int4,
        skill_nugget -> Int4,
        published -> Bool,
        skill_level -> Int4,
        priority -> Int4,
    }
}

joinable!(passwords -> users (id));
joinable!(sessions -> users (user_id));
joinable!(group_memberships -> users (user_id));
joinable!(group_memberships -> user_groups (group_id));
joinable!(anon_aliases -> users (user_id));
joinable!(anon_aliases -> user_groups (group_id));
joinable!(audio_files -> narrators (narrators_id));
joinable!(audio_files -> audio_bundles (bundle_id));
joinable!(words -> audio_bundles (audio_bundle));
joinable!(words -> skill_nuggets (skill_nugget));
joinable!(quiz_questions -> skill_nuggets (skill_id));
joinable!(question_answers -> quiz_questions (question_id));
joinable!(exercises -> skill_nuggets (skill_id));
joinable!(exercise_variants -> words (id));
joinable!(exercise_variants -> exercises (exercise_id));
joinable!(due_items -> users (user_id));
joinable!(pending_items -> users (user_id));
joinable!(pending_items -> audio_files (audio_file_id));
joinable!(q_asked_data -> pending_items (id));
joinable!(q_asked_data -> quiz_questions (question_id));
joinable!(q_asked_data -> question_answers (correct_qa_id));
joinable!(q_answered_data -> q_asked_data (id));
joinable!(q_answered_data -> question_answers (answered_qa_id));
joinable!(e_asked_data -> pending_items (id));
joinable!(e_asked_data -> exercises (exercise_id));
joinable!(e_asked_data -> exercise_variants (word_id));
joinable!(e_answered_data -> e_asked_data (id));
joinable!(w_asked_data -> pending_items (id));
joinable!(w_asked_data -> words (word_id));
joinable!(w_answered_data -> w_asked_data (id));
joinable!(question_data -> quiz_questions (question_id));
joinable!(question_data -> due_items (due));
joinable!(exercise_data -> exercises (exercise_id));
joinable!(exercise_data -> due_items (due));
joinable!(skill_data -> users (user_id));
joinable!(skill_data -> skill_nuggets (skill_nugget));
joinable!(user_metrics -> users (id));
joinable!(event_experiences -> users (user_id));
joinable!(event_experiences -> events (event_id));
joinable!(event_userdata -> users (user_id));
joinable!(event_userdata -> events (event_id));
joinable!(user_stats -> users (id));
joinable!(reset_email_secrets -> users (user_id));
joinable!(events -> user_groups (required_group));
joinable!(question_answers -> audio_bundles (a_audio_bundle));