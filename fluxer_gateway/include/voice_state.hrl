%% SPDX-License-Identifier: AGPL-3.0-or-later

-type voice_flags() :: #{
    self_mute := boolean(),
    self_deaf := boolean(),
    self_video := boolean(),
    self_stream := boolean(),
    is_mobile := boolean(),
    suppress := boolean()
}.
