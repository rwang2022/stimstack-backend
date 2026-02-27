# stimstack-backend

This project uses 
- caffeine decay models
- ?timeline simulation?
- crash time prediction
- sleep interference model 

Explanation of Code Structure: 

## Up Next 
- Intake schedule optimizer + constraint solver
    - suggest best times to take caffeine to maximize alertness, minimize crashes and poor sleep
    - you want to achieve certain goals like 
        - alertness >50% within 9am-5pm (depends on person)
        - max 400mg/day
        - no caffeine within 4 hours of bedtime (depends on person)
        - minimum 4 hours (depends on person) between doses
        - can use linear programming

- Personal sensitivity tuning (updating constants according to user idiosyncrasies)
    - half-life of caffeine varies by person 
    - sleep interference varies by person
    - lets you change half-life `k = ln(2) / half_life`
    - change sleep decay constant in `sleep_score`, tweak 50mg factor
    - learn from user feedback ("I felt sleept at 3pm" -> tweak model)

## Wording?
- Circadian rhythm modeling?
- Pharmacokinetic modeling?
- Tolerance modeling?

## Production
- PostgreSQL
- Redis
- Auth
- Docker

## Commands to remember
```bash
$ cargo clean
$ cargo watch -x run
$ cargo test
```