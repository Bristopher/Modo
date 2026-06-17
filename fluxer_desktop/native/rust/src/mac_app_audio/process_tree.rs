// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Info {
    pub pid: i32,
    pub parent_pid: i32,
    pub process_group_id: i32,
}

pub fn is_same_launch_tree_with_resolver(
    candidate_pid: i32,
    target_pid: i32,
    target_info: Option<Info>,
    mut resolver: impl FnMut(i32) -> Option<Info>,
) -> bool {
    if candidate_pid <= 0 || target_pid <= 0 {
        return false;
    }
    if candidate_pid == target_pid {
        return true;
    }

    let Some(mut current) = resolver(candidate_pid) else {
        return false;
    };
    if shares_process_group(current, target_pid, target_info) {
        return true;
    }

    for _ in 0..64 {
        let parent = current.parent_pid;
        if parent == target_pid {
            return true;
        }
        if parent <= 1 || parent == current.pid {
            return false;
        }
        let Some(next) = resolver(parent) else {
            return false;
        };
        current = next;
    }

    false
}

pub fn collect_related_pids_with_resolver(
    target_pid: i32,
    target_info: Option<Info>,
    candidates: &[i32],
    max_count: usize,
    mut resolver: impl FnMut(i32) -> Option<Info>,
) -> Vec<i32> {
    if target_pid <= 0 || max_count == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(candidates.len().max(1).min(max_count));
    append_pid(&mut out, max_count, target_pid);

    for &pid in candidates {
        if out.len() >= max_count {
            break;
        }
        if pid <= 0 || pid == target_pid {
            continue;
        }
        if !is_same_launch_tree_with_resolver(pid, target_pid, target_info, &mut resolver) {
            continue;
        }
        append_pid(&mut out, max_count, pid);
    }

    out
}

fn shares_process_group(candidate_info: Info, target_pid: i32, target_info: Option<Info>) -> bool {
    let candidate_group = candidate_info.process_group_id;
    if candidate_group <= 0 {
        return false;
    }
    if candidate_group == target_pid {
        return true;
    }
    target_info.is_some_and(|target| {
        target.process_group_id > 0 && candidate_group == target.process_group_id
    })
}

fn append_pid(pids: &mut Vec<i32>, max_count: usize, pid: i32) -> bool {
    if pids.contains(&pid) {
        return true;
    }
    if pids.len() >= max_count {
        return false;
    }
    pids.push(pid);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve_from<'a>(infos: &'a [Info]) -> impl FnMut(i32) -> Option<Info> + 'a {
        move |pid| infos.iter().copied().find(|info| info.pid == pid)
    }

    #[test]
    fn same_launch_tree_includes_direct_child() {
        let infos = [
            Info {
                pid: 100,
                parent_pid: 1,
                process_group_id: 100,
            },
            Info {
                pid: 101,
                parent_pid: 100,
                process_group_id: 100,
            },
        ];

        assert!(is_same_launch_tree_with_resolver(
            101,
            100,
            Some(infos[0]),
            resolve_from(&infos),
        ));
    }

    #[test]
    fn same_launch_tree_includes_process_group_peer() {
        let infos = [
            Info {
                pid: 200,
                parent_pid: 1,
                process_group_id: 200,
            },
            Info {
                pid: 201,
                parent_pid: 1,
                process_group_id: 200,
            },
        ];

        assert!(is_same_launch_tree_with_resolver(
            201,
            200,
            Some(infos[0]),
            resolve_from(&infos),
        ));
    }

    #[test]
    fn same_launch_tree_excludes_unrelated_process() {
        let infos = [
            Info {
                pid: 300,
                parent_pid: 1,
                process_group_id: 300,
            },
            Info {
                pid: 301,
                parent_pid: 1,
                process_group_id: 301,
            },
        ];

        assert!(!is_same_launch_tree_with_resolver(
            301,
            300,
            Some(infos[0]),
            resolve_from(&infos),
        ));
    }

    #[test]
    fn collect_related_pids_with_resolver_returns_selected_app_process_tree() {
        let infos = [
            Info {
                pid: 400,
                parent_pid: 1,
                process_group_id: 400,
            },
            Info {
                pid: 401,
                parent_pid: 400,
                process_group_id: 400,
            },
            Info {
                pid: 402,
                parent_pid: 401,
                process_group_id: 400,
            },
            Info {
                pid: 500,
                parent_pid: 1,
                process_group_id: 500,
            },
        ];
        let candidates = [500, 401, 402, 400];

        assert_eq!(
            vec![400, 401, 402],
            collect_related_pids_with_resolver(
                400,
                Some(infos[0]),
                &candidates,
                8,
                resolve_from(&infos),
            )
        );
    }

    #[test]
    fn cyclic_parent_chain_is_bounded() {
        let infos = [
            Info {
                pid: 600,
                parent_pid: 1,
                process_group_id: 600,
            },
            Info {
                pid: 601,
                parent_pid: 602,
                process_group_id: 601,
            },
            Info {
                pid: 602,
                parent_pid: 601,
                process_group_id: 602,
            },
        ];

        assert!(!is_same_launch_tree_with_resolver(
            601,
            600,
            Some(infos[0]),
            resolve_from(&infos),
        ));
    }
}
