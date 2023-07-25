const { createApp, ref } = Vue

const app = createApp({
    setup() {
      return {
        count: ref(0),
        version: ref({
            debug: false,
            name: 'no-name',
            ver: 'no-ver',
        }),
        currentId: ref({
            logId: 'no-log-id',
            blockId: 'no-block-id'
        })
      }
    },
    methods: {
        getVersion() {
            queueApi.version().then( d => {
                this.version.debug = d.debug
                this.version.name = d.crate_name
                this.version.ver = d.crate_ver
            });
        },
        getCurrentId() {
            queueApi.currentId().then( d => {
                this.currentId.logId = d.log_id
                this.currentId.blockId = d.block_id
            })
        }
    }    
  }).mount('#app')

window.addEventListener('load', (e)=>{
    app.getVersion()
    app.getCurrentId()
});
