#!/bin/sh


set -e 

psql - ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dname "$POSTGRES_DB" <<-EOSQL
            CREATE USER admin;
  EOSQL
