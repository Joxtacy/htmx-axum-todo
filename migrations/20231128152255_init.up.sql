--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

--
-- Name: todo; Type: TABLE; Schema: public; Owner: postgres
--
CREATE TABLE IF NOT EXISTS todo 
(
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    title text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    done boolean DEFAULT false NOT NULL,
    CONSTRAINT id_todo PRIMARY KEY ( id )
);

