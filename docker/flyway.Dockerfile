FROM flyway/flyway:12.0

COPY ./flyway/sql/ /flyway/sql/

CMD ["migrate"]
