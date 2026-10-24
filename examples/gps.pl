% Compare two supplied routes from Gent to Oostende using travel time, cost,
% belief, and comfort scores. Road connections are one-way; time is in
% seconds, and the remaining values are illustrative scores.
%
% The program adds time and cost along a route and multiplies its belief and
% comfort factors. In this two-route example, the recommended route is
% faster, cheaper, and higher on both other scores. The rules do not combine
% competing criteria into a general route-optimization score.

:- table route/2.
:- table duration/2.
:- table cost/2.
:- table belief/2.
:- table comfort/2.
:- table beats/2.
:- table 'recommended-route'/2.
:- table outcome/2.
location(traveller, gent).
destination(trip, oostende).
road([gent, brugge], 'drive-gent-brugge').
duration('drive-gent-brugge', 1500).
cost('drive-gent-brugge', 0.006).
belief('drive-gent-brugge', 0.96).
comfort('drive-gent-brugge', 0.99).
road([gent, kortrijk], 'drive-gent-kortrijk').
duration('drive-gent-kortrijk', 1600).
cost('drive-gent-kortrijk', 0.007).
belief('drive-gent-kortrijk', 0.96).
comfort('drive-gent-kortrijk', 0.99).
road([kortrijk, brugge], 'drive-kortrijk-brugge').
duration('drive-kortrijk-brugge', 1600).
cost('drive-kortrijk-brugge', 0.007).
belief('drive-kortrijk-brugge', 0.96).
comfort('drive-kortrijk-brugge', 0.99).
road([brugge, oostende], 'drive-brugge-oostende').
duration('drive-brugge-oostende', 900).
cost('drive-brugge-oostende', 0.004).
belief('drive-brugge-oostende', 0.98).
comfort('drive-brugge-oostende', 1).
name(['drive-gent-brugge', 'drive-brugge-oostende'], direct).
name(['drive-gent-kortrijk', 'drive-kortrijk-brugge', 'drive-brugge-oostende'], 'via-kortrijk').
label(direct, "Gent → Brugge → Oostende").
label('via-kortrijk', "Gent → Kortrijk → Brugge → Oostende").
duration([], 0).
cost([], 0).
belief([], 1).
comfort([], 1).

route([From, To], [Road]) :-
    road([From, To], Road).
route([From, To], [Road | Rest]) :-
    road([From, Via], Road),
    route([Via, To], Rest).
route(trip, Actions) :-
    location(traveller, From),
    destination(trip, To),
    route([From, To], Actions).
actions(Route, Actions) :-
    route(trip, Actions),
    name(Actions, Route).
is_a(Route, 'available-route') :-
    actions(Route, Actions).
duration([Action | Rest], _Calc1) :-
    duration(Action, Value),
    duration(Rest, Remaining),
    _Calc1 is (Value + Remaining).
cost([Action | Rest], _Calc1) :-
    cost(Action, Value),
    cost(Rest, Remaining),
    _Calc1 is (Value + Remaining).
belief([Action | Rest], _Calc1) :-
    belief(Action, Value),
    belief(Rest, Remaining),
    _Calc1 is (Value * Remaining).
comfort([Action | Rest], _Calc1) :-
    comfort(Action, Value),
    comfort(Rest, Remaining),
    _Calc1 is (Value * Remaining).
duration(Route, Value) :-
    actions(Route, Actions),
    duration(Actions, Value).
cost(Route, Value) :-
    actions(Route, Actions),
    cost(Actions, Value).
belief(Route, Value) :-
    actions(Route, Actions),
    belief(Actions, Value).
comfort(Route, Value) :-
    actions(Route, Actions),
    comfort(Actions, Value).
beats(Route, Other) :-
    is_a(Route, 'available-route'),
    is_a(Other, 'available-route'),
    duration(Route, Duration),
    duration(Other, OtherDuration),
    Duration < OtherDuration,
    cost(Route, Cost),
    cost(Other, OtherCost),
    Cost < OtherCost,
    belief(Route, Belief),
    belief(Other, OtherBelief),
    Belief > OtherBelief,
    comfort(Route, Comfort),
    comfort(Other, OtherComfort),
    Comfort > OtherComfort.
'recommended-route'(trip, Route) :-
    beats(Route, Other).
outcome(trip, Label) :-
    'recommended-route'(trip, Route),
    label(Route, Label).

% Answer and Reason
?- route(trip, What).
?- is_a(Who, 'available-route').
?- duration(direct, What).
?- cost(direct, What).
?- belief(direct, What).
?- comfort(direct, What).
?- duration('via-kortrijk', What).
?- cost('via-kortrijk', What).
?- belief('via-kortrijk', What).
?- comfort('via-kortrijk', What).
?- 'recommended-route'(trip, What).
?- outcome(trip, What).
?- why.

% Check
:- assertion((aggregate_all(count, is_a(Subject, 'available-route'), 2))).
:- assertion((route(trip, ['drive-gent-brugge', 'drive-brugge-oostende']))).
:- assertion((route(trip, ['drive-gent-kortrijk', 'drive-kortrijk-brugge', 'drive-brugge-oostende']))).
:- assertion((duration(direct, 2400))).
:- assertion((cost(direct, 0.01))).
:- assertion((belief(direct, 0.9408))).
:- assertion((comfort(direct, 0.99))).
:- assertion((duration('via-kortrijk', 4100))).
:- assertion((cost('via-kortrijk', 0.018))).
:- assertion((belief('via-kortrijk', 0.903168))).
:- assertion((comfort('via-kortrijk', 0.9801))).
:- assertion((beats(direct, 'via-kortrijk'))).
:- assertion((\+ (beats('via-kortrijk', direct)))).
:- assertion(('recommended-route'(trip, direct))).
:- assertion((\+ ('recommended-route'(trip, 'via-kortrijk')))).
:- assertion((outcome(trip, "Gent → Brugge → Oostende"))).
:- assertion((\+ (route([oostende, gent], ['drive-brugge-oostende', 'drive-gent-brugge'])))).
