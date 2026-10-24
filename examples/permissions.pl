% Derive read and write permissions from users' roles, then exclude suspended
% users. Alice is an editor, Bob is a viewer, and Carol is a suspended
% editor.
%
% The queries distinguish permissions suggested by a role from actions that
% remain allowed after exclusions. A user is treated as unsuspended when the
% completed program has no suspended fact for that user.

role(alice, editor).
role(bob, viewer).
role(carol, editor).
permits(editor, read).
permits(editor, write).
permits(viewer, read).
suspended(carol).

candidate(User, Action) :-
    role(User, Role),
    permits(Role, Action).
allowed(User, Action) :-
    candidate(User, Action),
    \+ suspended(User).

% Answer and Reason
?- allowed(User, Action).
?- why.
?- candidate(carol, Action).
?- why.

% Check
:- assertion(allowed(alice, read)).
:- assertion(allowed(alice, write)).
:- assertion(allowed(bob, read)).
:- assertion(\+ allowed(bob, write)).
:- assertion(candidate(carol, read)).
:- assertion(candidate(carol, write)).
:- assertion(\+ allowed(carol, read)).
:- assertion(\+ allowed(carol, write)).
:- assertion(aggregate_all(count, allowed(User, Action), 3)).
