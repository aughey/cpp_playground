#pragma GCC diagnostic ignored "-Wunused-parameter"

#include <stdint.h>

class IInterface
{
public:
    virtual void create_thing(uint32_t create_id) = 0;
    virtual void destroy_thing(uint32_t destroy_id) = 0;
};

class IActor
{
public:
    virtual void on_frame(IInterface &interface) = 0;
    virtual void on_thing_created(uint32_t create_id, uint32_t entity_id) = 0;
    virtual void on_thing_destroyed(uint32_t create_id, uint32_t entity_id) = 0;
};

class MyActor : IActor
{
public:
    void on_frame(IInterface &interface) override
    {
    }

    void on_thing_created(uint32_t create_id, uint32_t entity_id) override
    {
    }

    void on_thing_destroyed(uint32_t create_id, uint32_t entity_id) override
    {
    }
};