from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable, Optional, Set, Tuple


@dataclass(frozen=True)
class LayoutLabels:
    session_list_names: Set[str]
    session_search_names: Set[str]
    message_list_names: Set[str]
    send_button_names: Set[str]
    navigation_button_names: Set[str]


@dataclass
class MainControls:
    navigation: Optional[object]
    session: Optional[object]
    chat: Optional[object]


DEFAULT_MAX_DEPTH = 14
SUBTREE_SCAN_DEPTH = 6


def discover_main_controls(root: object, labels: LayoutLabels) -> MainControls:
    session = _find_session_container(root, labels, DEFAULT_MAX_DEPTH)
    chat = _find_chat_container(root, labels, DEFAULT_MAX_DEPTH)
    navigation = _find_navigation_container(root, labels, DEFAULT_MAX_DEPTH)
    return MainControls(navigation=navigation, session=session, chat=chat)


def _find_session_container(root: object, labels: LayoutLabels, max_depth: int) -> Optional[object]:
    session_list = _find_first_by_type_and_name(
        root, "ListControl", labels.session_list_names, max_depth
    )
    if session_list:
        candidate = _find_ancestor_with(
            session_list,
            lambda ctrl: _looks_like_session_container(ctrl, labels),
        )
        if candidate:
            return candidate

    search_box = _find_first_by_type_and_name(
        root, "EditControl", labels.session_search_names, max_depth
    )
    if search_box:
        candidate = _find_ancestor_with(
            search_box,
            lambda ctrl: _looks_like_session_container(ctrl, labels),
        )
        if candidate:
            return candidate

    return _find_first_control_matching(
        root, lambda ctrl: _looks_like_session_container(ctrl, labels), max_depth
    )


def _find_chat_container(root: object, labels: LayoutLabels, max_depth: int) -> Optional[object]:
    message_list = _find_first_by_type_and_name(
        root, "ListControl", labels.message_list_names, max_depth
    )
    if message_list:
        candidate = _find_ancestor_with(
            message_list,
            lambda ctrl: _looks_like_chat_container(ctrl, labels),
        )
        if candidate:
            return candidate

    send_button = _find_first_by_type_and_name(
        root, "ButtonControl", labels.send_button_names, max_depth
    )
    if send_button:
        candidate = _find_ancestor_with(
            send_button,
            lambda ctrl: _looks_like_chat_container(ctrl, labels),
        )
        if candidate:
            return candidate

    return _find_first_control_matching(
        root, lambda ctrl: _looks_like_chat_container(ctrl, labels), max_depth
    )


def _find_navigation_container(root: object, labels: LayoutLabels, max_depth: int) -> Optional[object]:
    nav_button = _find_first_by_type_and_name(
        root, "ButtonControl", labels.navigation_button_names, max_depth
    )
    if nav_button:
        candidate = _find_ancestor_with(
            nav_button,
            lambda ctrl: _count_controls_by_name(ctrl, "ButtonControl", labels.navigation_button_names)
            >= 3,
        )
        if candidate:
            return candidate
        return _safe_parent(nav_button)

    return _find_first_control_matching(
        root,
        lambda ctrl: _count_controls_by_name(ctrl, "ButtonControl", labels.navigation_button_names)
        >= 3,
        max_depth,
    )


def _looks_like_session_container(control: object, labels: LayoutLabels) -> bool:
    return _has_descendant_by_type_and_name(
        control, "EditControl", labels.session_search_names
    ) and _has_descendant_by_type_and_name(
        control, "ListControl", labels.session_list_names
    )


def _looks_like_chat_container(control: object, labels: LayoutLabels) -> bool:
    has_edit = _has_descendant_by_type(control, "EditControl")
    has_message_list = _has_descendant_by_type_and_name(
        control, "ListControl", labels.message_list_names
    )
    has_send = _has_descendant_by_type_and_name(
        control, "ButtonControl", labels.send_button_names
    )
    return has_edit and (has_message_list or has_send)


def _has_descendant_by_type(control: object, control_type: str) -> bool:
    return _subtree_has(
        control, lambda ctrl: _control_type(ctrl) == control_type, SUBTREE_SCAN_DEPTH
    )


def _has_descendant_by_type_and_name(
    control: object, control_type: str, names: Set[str]
) -> bool:
    names = {name for name in names if name}
    if not names:
        return False
    return _subtree_has(
        control,
        lambda ctrl: _control_type(ctrl) == control_type and _control_name(ctrl) in names,
        SUBTREE_SCAN_DEPTH,
    )


def _count_controls_by_name(control: object, control_type: str, names: Set[str]) -> int:
    names = {name for name in names if name}
    if not names:
        return 0
    count = 0
    for ctrl, _ in _iter_controls(control, SUBTREE_SCAN_DEPTH):
        if _control_type(ctrl) == control_type and _control_name(ctrl) in names:
            count += 1
    return count


def _find_first_by_type_and_name(
    root: object, control_type: str, names: Set[str], max_depth: int
) -> Optional[object]:
    names = {name for name in names if name}
    if not names:
        return None
    return _find_first_control_matching(
        root,
        lambda ctrl: _control_type(ctrl) == control_type and _control_name(ctrl) in names,
        max_depth,
    )


def _find_first_control_matching(
    root: object, predicate, max_depth: int
) -> Optional[object]:
    for ctrl, _ in _iter_controls(root, max_depth):
        if predicate(ctrl):
            return ctrl
    return None


def _find_ancestor_with(control: object, predicate, max_levels: int = 10) -> Optional[object]:
    current = control
    steps = 0
    while current is not None and steps < max_levels:
        if predicate(current):
            return current
        current = _safe_parent(current)
        steps += 1
    return None


def _subtree_has(control: object, predicate, max_depth: int) -> bool:
    for ctrl, _ in _iter_controls(control, max_depth):
        if predicate(ctrl):
            return True
    return False


def _iter_controls(control: object, max_depth: int) -> Iterable[Tuple[object, int]]:
    if hasattr(control, "Walk"):
        try:
            for item in control.Walk(includeTop=True, maxDepth=max_depth):
                yield item
            return
        except Exception:
            pass

    stack = [(control, 0)]
    while stack:
        node, depth = stack.pop()
        yield node, depth
        if depth >= max_depth:
            continue
        children = []
        if hasattr(node, "GetChildren"):
            try:
                children = node.GetChildren()
            except Exception:
                children = []
        for child in reversed(children):
            stack.append((child, depth + 1))


def _control_type(control: object) -> str:
    value = getattr(control, "ControlTypeName", "")
    return value or ""


def _control_name(control: object) -> str:
    value = getattr(control, "Name", "")
    if isinstance(value, str):
        return value.strip()
    return ""


def _safe_parent(control: object) -> Optional[object]:
    if not hasattr(control, "GetParentControl"):
        return None
    try:
        return control.GetParentControl()
    except Exception:
        return None
